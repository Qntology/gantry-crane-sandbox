# 공개 저장소에 있는 실데이터를 data\raw 로 받는다 (Windows PowerShell). 라이선스는 docs\data-sources.md 참고.
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
New-Item -ItemType Directory -Force -Path data\raw | Out-Null
function Clone($url, $dir) {
  if (Test-Path "data\raw\$dir") { Write-Host "있음: data\raw\$dir"; return }
  git clone --depth 1 $url "data\raw\$dir"
}
Clone "https://github.com/jrgvicente/Crane3DSim" "Crane3DSim"
Clone "https://github.com/WangBiaoXJTU/xjtu-sy-bearing-datasets" "xjtu-sy"
if (Get-Command kaggle -ErrorAction SilentlyContinue) {
  kaggle datasets download -d wkirgsn/electric-motor-temperature -p data\raw\electric-motor-temperature --unzip
} else {
  Write-Host "kaggle CLI 가 없습니다: pip install kaggle 후 %USERPROFILE%\.kaggle\kaggle.json 을 두고 다시 실행하세요"
}
Write-Host ""
Write-Host "직접 받아야 하는 자료: docs\data-sources.md 의 '우선 확보 순서' 참고 (Aalto IEEE DataPort, Mendeley, Paderborn KAt, Felixstowe RTG)"
