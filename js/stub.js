/*
 Implements simple Bumaga engine for development and prototyping of HTML view in browser.
 */

function traverse(node, value) {
    if (node.nodeType !== 1) {
        node.textContent = node.textContent.replaceAll(REGEX_PIPE_GLOBAL, (substring, key) => {
            return getValue(value, key)
        });
        return;
    }
    if (node.getAttribute("repeated") === "true") {
        return;
    }
    let parent = node.parentNode;
    let newAttributes = [];
    for (let attribute of node.attributes) {
        if (attribute.name.startsWith("*")) {
            let key = attribute.name.substring(1);
            let [count, binder] = attribute.value.split(" ");
            count = parseInt(count);
            binder = parseBinder(binder);
            parent.removeChild(node);
            node.attributes.removeNamedItem(attribute.name);
            for (let item of getValue(value, binder)) {
                value[key] = item;
                let child = node.cloneNode(true);
                traverse(child, value);
                child.setAttribute("repeated", "")
                parent.appendChild(child);
            }
        }
        if (attribute.name.startsWith("+")) {
            let key = attribute.name.substring(1);
            let binder = parseBinder(attribute.value);
            value[key] = getValue(value, binder)
        }
        if (attribute.name.startsWith("^")) {
            let handler = attribute.name.substring(1);
            let [event, ...args] = attribute.value.split(" ");
            let resolved = [];
            for (let arg of args) {
                if (arg === "this") {
                    resolved.push("<this>");
                } else {
                    resolved.push(getValue(value, parseBinder(arg)))
                }
            }
            node.addEventListener(handler.substring(2), () => {
                console.log(event, resolved);
            })
        }
        if (attribute.name.startsWith("?")) {
            let binder = parseBinder(attribute.value);
            if (!getValue(value, binder)) {
                parent.removeChild(node);
                return;
            }
        }
        if (attribute.name.startsWith("!")) {
            let binder = parseBinder(attribute.value);
            if (getValue(value, binder)) {
                parent.removeChild(node);
                return;
            }
        }
        if (attribute.name.startsWith("@")) {
            let key = attribute.name.substring(1);
            let attr = attribute.value.replaceAll(REGEX_PIPE_GLOBAL, (substring, path) => {
                return getValue(value, path)
            });
            newAttributes.push([key, attr]);
        }
        if (attribute.name.startsWith("#")) {
            let key = attribute.name.substring(1);
            let binder = parseBinder(attribute.value);
            if (getValue(value, binder)) {
                newAttributes.push([key, ""]);
            }
        }
    }
    for (let setter of newAttributes) {
        let [key, value] = setter;
        node.setAttribute(key, value);
    }
    for (let child of node.childNodes) {
        traverse(child, value);
    }
}


let REGEX_PIPE_GLOBAL = /{([\w._]+)}/g;
let REGEX_PIPE = /{([\w._]+)}/;

function parseBinder(binder) {
    return REGEX_PIPE.exec(binder)[1];
}

function getValue(value, path) {
    return getValueRecursive(value, path.split("."));
}

function getValueRecursive(value, path) {
    let key = path.shift();
    if (path.length > 0) {
        return getValueRecursive(value[key], path);
    } else {
        return value[key]
    }
}

window.onload = () => {
    traverse(document.body, VALUE)
}