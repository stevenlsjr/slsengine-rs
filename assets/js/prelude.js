(() => {
    let Module;
    Module = window.Module || {};
    let canvas = document.createElement('canvas');
    document.body.appendChild(canvas);

    window.Module = Module;
})();