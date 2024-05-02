"use strict"

import { invoke } from "@tauri-apps/api";
import { DEFAULT_MATH_CONFIG, expressions } from "./equations";
import { functionSet } from "./functions";

//@ts-ignore this is a ide error, because of old JQuery library
const MQ = MathQuill.getInterface(2);

export const variableBoxes: Map<string, VariableBox> = new Map();

export class VariableBox {
    name: string;
    code: string;
    htmlElement: HTMLElement;
    mathField: any;
    undefVarsBar: UndefVariableBar;

    static createNew(name: string): VariableBox {
        const box = new VariableBox(name);
        variableBoxes.set(name, box);
        expressions.forEach(e => e.refresh());
        
        const conf = DEFAULT_MATH_CONFIG;
        conf.handlers = { edit: box.refresh };
        box.mathField.config(conf);
        
        box.focus();

        return box;
    }

    constructor(name: string) {
        this.name = name;
        this.code = "";
        this.undefVarsBar = new UndefVariableBar([]);

        this.htmlElement = this.#createVariableBox();
    }

    #createVariableBox(): HTMLElement {
        const box = document.createElement('div');
        box.className = 'expr';
    
        const container = document.createElement('div');
        container.className = 'expr-container';

        const btn = document.createElement('div');
        btn.className = 'expr-button';
        container.append(btn);

        const prefix = document.createElement('span');
        prefix.className = 'math-field';
        prefix.textContent = this.name + "=";
        MQ.StaticMath(prefix);
        container.append(prefix);

        const span = document.createElement('span');
        span.className = 'math-field';
        this.mathField = MQ.MathField(span);
        container.append(span);

        const close = document.createElement('button');
        close.insertAdjacentHTML('beforeend', '<span><i class="fa-solid fa-xmark"></i></span>');
        close.className = 'close-button';
        close.addEventListener('click', () => {
            this.htmlElement.remove();
            variableBoxes.delete(this.name);
            expressions.forEach(e => e.refresh());
            variableBoxes.forEach(e => e.refresh());
        });
        container.append(close);
        
        box.appendChild(container);
        
        box.appendChild(this.undefVarsBar.html);

        return box;
    }

    focus() {
        this.mathField.focus();
    }

    getVariables() {
        return new Set([...this.htmlElement.getElementsByTagName('var')]
                .filter(e => !e.classList.contains('mq-operator-name'))
                .map(e => e.textContent)
                .flatMap(e => e? e:[]));
    }

    refresh = async () => {
        const latex = this.mathField.latex()!;
        this.code = latex;
        
        const vars = this.getVariables();
        if(vars.has('x') || vars.has('y')) {
            this.writeError(new Error("A variable can't have x nor y because it has to be constant"));
            return;
        }

        const undef = [...vars].filter(e => !e.includes("xye") && !variableBoxes.has(e));
        if(undef.some(e => functionSet.has(e))) {
            this.writeError(new Error("A variable can't be named like a function"));
            return;
        }
        this.undefVarsBar.ofArray(undef);
    
        try {
            console.time('variable call');
            await invoke('add_variable', { name: this.name, content: this.code });
            expressions.forEach(e => e.refresh());
            console.timeEnd('variable call');
        } catch(error) {
            console.warn(error);
            this.writeError(error);
            return;
        }

        this.resetError();
    }

    resetError() {
        const e = this.htmlElement.querySelector(".expr-button");

        const elems = e?.getElementsByTagName('i');
        if(elems) for(let i of elems) i.remove();
    }

    writeError(tooltip: any) {
        const e = this.htmlElement.querySelector(".expr-button");        

        const elems = e?.getElementsByTagName('i');
        if(elems) for(let i of elems) i.remove();

        e?.insertAdjacentHTML('beforeend', `<i class="fa-solid fa-triangle-exclamation error-box-inverted" title="${tooltip}"></i>`);
    }
}

export class UndefVariableBar {
    #elements: Map<string, HTMLElement>;
    html: HTMLElement;

    constructor(array: string[]) {
        this.html = document.createElement('div');
        this.html.className = 'expr-variable-bar';

        this.#elements = new Map();
        this.ofArray(array);
    }

    #createVariableButton(name: string): HTMLElement {
        const btn = document.createElement('button');
        btn.className = 'expr-variable-btn';
        btn.textContent = name;

        btn.onclick = () => {
            this.delete(name);

            const box = VariableBox.createNew(name);
            const sidebar = document.getElementById('sidebar');
            sidebar?.appendChild(box.htmlElement);
            box.focus();
        }

        return btn;
    }

    ofArray(array: string[]) {
        const toDelete = [...this.#elements.keys()].filter(e => !array.includes(e));
        const toAdd = array.filter(e => !this.#elements.has(e));

        for(const v of toDelete)
            this.delete(v);

        for(const v of toAdd)
            this.add(v);
    }

    delete(name: string) {
        const btn = this.#elements.get(name);
        btn?.remove();
        this.#elements.delete(name);
    }

    add(name: string) {
        const btn = this.#createVariableButton(name);
        this.#elements.set(name, btn);
        this.html.appendChild(btn);
    }
}