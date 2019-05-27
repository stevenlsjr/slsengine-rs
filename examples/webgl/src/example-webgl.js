import("../../../slsengine-backend-webgl/pkg/slsengine_backend_webgl").then(
    wasm => {
        const canvas = document.createElement("canvas");
        document.body.appendChild(canvas);
        wasm.run_renderer(canvas);
    }
);
