/*
 Implements simple Bumaga engine for development and prototyping of HTML view in browser.
 */

let repeat_pattern = /\{(\w+)\}\*(\d+)/g;
let attribute_pattern = /\[([\w-_]+)\]/g;
let if_pattern = /\?/g;
let else_pattern = /\!/g;
let pipe_pattern = /\{([\w._]+)\}/g;
let callback_pattern = /\[([\w-_]+)\]~([\w-_]+)/;

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

function traverse(node, value) {
    if (node.nodeType !== 1) {
        node.textContent = node.textContent.replaceAll(pipe_pattern, (substring, key) => {
            return getValue(value, key)
        });
        return;
    }

    if (node.getAttribute("rendered") === "true") {
        return;
    }

    let parent = node.parentNode;
    for (let attribute of node.attributes) {
        if (if_pattern.exec(attribute.name) != null) {
            let getter = pipe_pattern.exec(attribute.value)[1];
            let test = getValue(value, getter);
            if (!test) {
                parent.removeChild(node);
                return;
            }
        }
        if (else_pattern.exec(attribute.name) != null) {
            let getter = pipe_pattern.exec(attribute.value)[1];
            let test = getValue(value, getter);
            if (!!test) {
                parent.removeChild(node);
                return;
            }
        }

        let repeat = repeat_pattern.exec(attribute.name);
        if (repeat != null) {
            let getter = repeat[1];
            let count = repeat[2];
            let items = pipe_pattern.exec(attribute.value)[1];
            parent.removeChild(node);
            node.attributes.removeNamedItem(attribute.name);
            for (let item of getValue(value, items)) {
                value[getter] = item;
                let child = node.cloneNode(true);
                parent.appendChild(child);
                traverse(child, value);
                child.setAttribute("rendered", "true")
            }
            // todo: count
            // todo: override
            delete value[getter];
            return;
        }
    }

    let attributes = [];
    for (let attribute of node.attributes) {
        let attr = attribute_pattern.exec(attribute.name);
        if (attr != null) {
            let name = attr[1];
            let getter = pipe_pattern.exec(attribute.value)[1];
            attributes.push([name, getValue(value, getter)]);
        }
        let callback = callback_pattern.exec(attribute.name);
        if (callback != null) {
            let event = callback[1];
            let handler = callback[2];
            let expr = attribute.value;
            attributes.push([event, `emit("${handler}${expr}")`])
        }
    }
    for (let setter of attributes) {
        let [key, value] = setter;
        node.setAttribute(key, value);
    }

    for (let child of node.childNodes) {
        traverse(child, value);
    }
}

window.onload = () => {
    console.log('BEGIN', VALUE);
    traverse(document.body, VALUE)
}

function emit(...args) {
    console.log('output', args)
}