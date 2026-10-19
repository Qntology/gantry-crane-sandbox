//! 모터 프리셋의 권선 시간상수를 SEW S3 출력 배수표에 맞춘다 (정지 냉각 0.22 고정)
use crane_model::calib::{fit_s3, ThermalKnob};
use crane_model::motor::*;
fn main() {
    for mut m in presets() {
        m.thermal.standstill_cooling = 0.22;
        if m.name == "im_0p75kw_4p" { m.thermal.tau_frame_s = 500.0; }
        if m.name == "im_1p5kw_4p" { m.thermal.tau_frame_s = 600.0; }
        let f = fit_s3(&m, &SEW_S3_FACTORS, 600.0, &[ThermalKnob::TauWinding], 400);
        println!("{:14} tau_w={:6.1}s tau_f={:6.0}s rms={:.4}  {:?}", f.motor, f.tau_winding_s, f.tau_frame_s, f.rms_error,
            f.points.iter().map(|(c,k,mk)| format!("{:.0}%:{:.2}/{:.3}", c*100.0, k, mk)).collect::<Vec<_>>());
    }
}
