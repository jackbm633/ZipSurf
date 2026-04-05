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

globalThis.document = {
    querySelectorAll: (s) => {
        let handles = rustQuerySelectorAll(s);
        return handles.map(h => new Node(h));
    }
};