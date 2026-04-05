// Ensure console exists globally
globalThis.console = {
    log: (...args) => {
        // Join arguments with a space, similar to standard console behavior
        const message = args.map(arg =>
            typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
        ).join(' ');

        rustLog(message);
    },
    error: (...args) => {
        rustLog("ERROR: " + args.join(' '));
    }
};
function Node(handle) { this.handle = handle; }

globalThis.document = {
    querySelectorAll: (s) => {
        let handles = rustQuerySelectorAll(s);
        return handles.map(function(h) { return new Node(h) });
    }
};

Node.prototype.getAttribute = function(attr) {
    return rustGetAttribute(this.handle, attr);
}

LISTENERS = {}

Node.prototype.addEventListener = function(type, listener) {
    if (!LISTENERS[this.handle]) LISTENERS[this.handle] = {};
    var dict = LISTENERS[this.handle];
    if (!dict[type]) dict[type] = [];
    var list = dict[type];
    list.push(listener);
}