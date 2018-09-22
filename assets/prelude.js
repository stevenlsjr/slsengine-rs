
function createRoot() {
    const elt = document.createElement('div')
    elt.id = 'app-root'
    document.body.appendChild(elt);
    return elt;
}

const root = document.querySelector("#app-root") ||
    createRoot();