use zeroflow::run;

fn main() {
    // WebKitGTK + KWin/Wayland: protocol error 71 unless the dmabuf renderer
    // is disabled (AMDGPU/NVIDIA compositing path unsupported here).
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    run();
}