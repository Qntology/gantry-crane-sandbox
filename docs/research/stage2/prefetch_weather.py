"""Prefetch Open-Meteo data into the crane sandbox weather cache (raw API JSON, same names the Rust loader uses).

usage: python -I prefetch_weather.py <port_sites.json> <cache_dir> <log_file>

Pacing follows the Open-Meteo free tier (600/min, 5000/h, 10000/day) with the documented weighting:
calls = max(1, variables/10) * max(1, days/14).
"""
import json
import os
import sys
import time
import urllib.request
import datetime as dt

ARCHIVE = "https://archive-api.open-meteo.com/v1/archive"
AIR = "https://air-quality-api.open-meteo.com/v1/air-quality"
MARINE = "https://marine-api.open-meteo.com/v1/marine"
DAILY_VARS = "temperature_2m_mean,temperature_2m_max,temperature_2m_min,precipitation_sum,snowfall_sum,wind_speed_10m_mean,wind_speed_10m_max,wind_gusts_10m_max,relative_humidity_2m_mean,wind_direction_10m_dominant"
HOURLY_VARS = "temperature_2m,relative_humidity_2m,dew_point_2m,precipitation,snowfall,wind_speed_10m,wind_direction_10m,wind_gusts_10m"
MODEL = "era5_seamless"

sites_path, cache, log_path = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(cache, exist_ok=True)
sites = json.load(open(sites_path, encoding="utf-8"))["sites"]
history = []  # (time, weight)
total = 0.0


def log(msg):
    line = f"{dt.datetime.now():%H:%M:%S} {msg}"
    print(line, flush=True)
    with open(log_path, "a", encoding="utf-8") as f:
        f.write(line + "\n")


def days_between(a, b):
    return (dt.date.fromisoformat(b) - dt.date.fromisoformat(a)).days + 1


def weight(nvars, start, end):
    return max(1.0, nvars / 10.0) * max(1.0, days_between(start, end) / 14.0)


def pace(w):
    while True:
        now = time.time()
        h = [(t, x) for t, x in history if now - t < 3600]
        history[:] = h
        m = sum(x for t, x in h if now - t < 60)
        hr = sum(x for _, x in h)
        if m + w <= 550 and hr + w <= 4700:
            return
        time.sleep(15)


def key(kind, lat, lon, start, end):
    return os.path.join(cache, f"{kind}_{lat:.3f}_{lon:.3f}_{start}_{end}.json")


def fetch(kind, base, lat, lon, start, end, params, nvars):
    global total
    path = key(kind, lat, lon, start, end)
    if os.path.exists(path):
        log(f"skip {os.path.basename(path)}")
        return True
    w = weight(nvars, start, end)
    if total + w > 9800:
        log(f"daily budget reached, stop before {os.path.basename(path)}")
        return False
    pace(w)
    url = f"{base}?latitude={lat}&longitude={lon}&start_date={start}&end_date={end}&{params}&timezone=auto"
    for attempt in range(3):
        try:
            with urllib.request.urlopen(url, timeout=300) as r:
                body = r.read()
            data = json.loads(body)
            if data.get("error"):
                log(f"error {kind} {lat},{lon}: {data.get('reason')}")
                return True
            tmp = path + ".tmp"
            with open(tmp, "wb") as f:
                f.write(body)
            os.replace(tmp, path)
            history.append((time.time(), w))
            total += w
            log(f"ok {os.path.basename(path)} w={w:.0f} total={total:.0f} bytes={len(body)}")
            return True
        except Exception as e:  # noqa: BLE001
            log(f"retry {attempt + 1} {kind} {lat},{lon}: {e}")
            history.append((time.time(), w))
            time.sleep(60)
    return True


jobs = []
for s in sites:
    lat, lon = s["lat"], s["lon"]
    if s["kind"] == "kma_station":
        for a, b in [(1991, 2000), (2001, 2010), (2011, 2020)]:
            jobs.append((0, "daily-era5", ARCHIVE, lat, lon, f"{a}-01-01", f"{b}-12-31", f"daily={DAILY_VARS}&models={MODEL}&wind_speed_unit=ms", 10))
        jobs.append((1, "hourly-era5", ARCHIVE, lat, lon, "2022-01-01", "2023-12-31", f"hourly={HOURLY_VARS}&models={MODEL}&wind_speed_unit=ms", 8))
    else:
        jobs.append((0, "daily-era5", ARCHIVE, lat, lon, "2014-01-01", "2023-12-31", f"daily={DAILY_VARS}&models={MODEL}&wind_speed_unit=ms", 10))
        jobs.append((1, "hourly-era5", ARCHIVE, lat, lon, "2023-01-01", "2023-12-31", f"hourly={HOURLY_VARS}&models={MODEL}&wind_speed_unit=ms", 8))
        jobs.append((2, "air", AIR, lat, lon, "2023-01-01", "2023-12-31", "hourly=sulphur_dioxide", 1))
        if "marine_lat" in s:
            jobs.append((2, "marine", MARINE, s["marine_lat"], s["marine_lon"], "2023-01-01", "2023-12-31", "hourly=wave_height", 1))

jobs.sort(key=lambda j: j[0])
log(f"{len(jobs)} jobs, est. weight {sum(weight(j[8], j[5], j[6]) for j in jobs):.0f}")
for j in jobs:
    if not fetch(j[1], j[2], j[3], j[4], j[5], j[6], j[7], j[8]):
        break
log("done")
