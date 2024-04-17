"use strict"

import { emit } from "@tauri-apps/api/event";
import numeral from "numeral";
import { UndefVariableBar, variableBoxes } from "./variables";

//@ts-ignore this is a ide error, because of old JQuery library
const MQ = MathQuill.getInterface(2);

export const expressions : Map<number, EquationBox> = new Map();

export enum EditAction {
    ADD, DELETE, REFRESH
}
export interface EditPayload {
    latex: string,
    id: number,
    action: EditAction,
}

export const EDIT_EMIT_CODE: string = 'edited';
const AUTO_FUNCTIONS = 'sin cos tan sec csc cosec cotan mod floor abs ceil log ln';
const AUTO_COMMANDS = 'pi theta sqrt sum rho phi lambda';

export const DEFAULT_MATH_CONFIG = {
    spaceBehavesLikeTab: true,
    autoCommands: AUTO_COMMANDS,
    autoOperatorNames: AUTO_FUNCTIONS,
    handlers: <any> null,
};

export class EquationBox {
    static currNumber = 0;
    static currHue = 0;
    static createNew(): EquationBox {
        const eq = new EquationBox(this.currNumber, this.currHue);
        expressions.set(this.currNumber, eq);

        const conf = DEFAULT_MATH_CONFIG;
        conf.handlers = { edit: eq.refresh };
        eq.mathField.config(conf);

        this.currNumber++;
        this.currHue = (this.currHue + 49) % 360;

        return eq;
    }
    
    
    number: number;
    color: string;
    visible = true;
    error = false;
    htmlElement: HTMLElement;
    mathField?: any;
    latexLength: number;
    solutionBox?: HTMLElement;
    undefVarsBar: UndefVariableBar;
    code?: string;

    constructor(number: number, hue: number) {
        this.number = number;
        this.color = `hsl(${hue} 69% 69%)`;
        this.latexLength = 0;
        this.undefVarsBar = new UndefVariableBar([]);
        
        this.htmlElement = this.#createEqBox();
    }

    #createEqBox() {
        this.htmlElement = document.createElement('div');
        this.htmlElement.className = 'expr';
        this.htmlElement.id = 'eq-bar-' + this.number;
    
        const container = document.createElement('div');
        container.className = 'expr-container';
    
        const btn = document.createElement('div');
        btn.className = 'expr-button';
        btn.id = 'expr-button-' + this.number;
        btn.style.background = this.color;
        btn.style.boxShadow = '0px 0px 5px 3px ' + this.color;
        btn.addEventListener('click', () => {
            this.visible = !this.visible;
            btn.style.background = this.visible? this.color : '#1d1d1d';
            this.toggleError();
        });
        container.append(btn);
    
        const span = document.createElement('span');
        span.className = 'math-field';
        this.mathField = MQ.MathField(span);
        container.append(span);
    
        const close = document.createElement('button');
        close.insertAdjacentHTML('beforeend', '<span><i class="fa-solid fa-xmark"></i></span>');
        close.className = 'close-button';
        close.addEventListener('click', () => {
            this.htmlElement.remove();
            expressions.delete(this.number);
        });
        container.append(close);
    
        this.htmlElement.append(container);
    
        const exprBottom = document.createElement('div');
        exprBottom.className = 'expr-bottom';

        this.solutionBox = document.createElement('span');
        this.solutionBox.className = 'solution-box';
        exprBottom?.appendChild(this.solutionBox);
        this.hideSolutionBox();

        exprBottom.append(this.undefVarsBar.html);

        this.htmlElement.append(exprBottom);

        return this.htmlElement;
    }
    
    refresh = async () => {
        const old = this.latexLength;
        const len = this.mathField.latex().length;

        let actionType: EditAction;
        if(len == old) actionType = EditAction.REFRESH;
        else if(len > old) actionType = EditAction.ADD;
        else actionType = EditAction.REFRESH; 

        this.latexLength = len;

        await emit(EDIT_EMIT_CODE, <EditPayload> {
            latex: this.mathField.latex(),
            id: this.number,
            action: actionType,
        })
    }

    focus() {
        this.mathField.focus();
    }

    toggleError() {
        const e = this.htmlElement.querySelector(".expr-button");

        const elems = e?.getElementsByTagName('i');
        if(elems) for(let i of elems) i.remove();
        
        if(this.error) {
            e?.insertAdjacentHTML('beforeend', `<i class="fa-solid fa-triangle-exclamation ${
                this.visible? "error-box":"error-box-inverted"
            }"></i>`);
        }
    }
    
    writeError(tooltip: any) {
        const e = this.htmlElement.querySelector(".expr-button");        
        this.error = true;

        const elems = e?.getElementsByTagName('i');
        if(elems) for(let i of elems) i.remove();

        e?.insertAdjacentHTML('beforeend', `<i class="fa-solid fa-triangle-exclamation ${
            this.visible? "error-box":"error-box-inverted"
        }" title="${tooltip}"></i>`);
    }

    /**
     * @throws An error if the function is named x, y or e
     * @returns The character of the name of the function or null otherwise
     */
    functionCharacter(): string | null {
        const fn = this.mathField.latex().match('[A-Za-z]\\\\left\\([x-y]\\\\right\\)=');
        if(!fn) 
            return null;
        
        const name = fn[0][0];
        if("xye".includes(name))
            throw Error(`A function can't be named ${name}, it's a reserved character`);
        if(variableBoxes.has(name))
            throw Error(`There is already a variable with that name: ${name}`);
        return name;
    }

    getVariables() {
        const fnName = this.functionCharacter();

        const vars = Array.from(this.htmlElement.getElementsByTagName('var'))
                .filter(e => !e.classList.contains('mq-operator-name'))
                .map(e => e.textContent)
                .flatMap(e => e? e:[]);
        
        if(fnName)
            vars.splice(0, 1);

        return new Set(vars);
    }

    writeFunctionBrackets() {
        const cursor = this.htmlElement.querySelector(".mq-cursor");
        const prev = cursor?.previousElementSibling;
        if(prev && prev.classList.contains('mq-operator-name')) {
            this.mathField?.typedText('(');
        }
    }

    setSolutionValue(n: number) {
        if(!this.solutionBox) 
            throw Error("No solution box");
        this.solutionBox.style.display = 'inline';
        this.solutionBox.textContent = numeral(n).format('0[.][000000]');
    }

    hideSolutionBox() {
        if(!this.solutionBox) 
            throw Error("No solution box");
        this.solutionBox.style.display = 'none';
    }

    showUndefinedVariables(undefVars: string[]) {
        this.undefVarsBar.ofArray(undefVars);
    }
}