"use strict"

import { invoke } from "@tauri-apps/api";
import { EditAction, EquationBox, expressions } from "./equations";
import { Response } from "./main";
import { draw } from "./renderer";

export const functionSet: Map<string, number> = new Map();

export const addFunction = async (fnName: string, latex: string, eq: EquationBox, action: EditAction) => {
    const code = latex.substring(latex.indexOf('=')+1);
    const unknown = Math.min(latex.indexOf('x')>0? latex.indexOf('x'):1e9, latex.indexOf('y')>0? latex.indexOf('y'):1e9);

    try {
        fnName += latex[unknown]!;
        console.log(fnName)
        const response = <Response> await invoke('add_function', { name: fnName, content: code });

        if(action != EditAction.REFRESH)
            for(let id of expressions.keys())
                if(id != eq.number)
                    expressions.get(id)?.refresh(); //FIXME: This is shit for performance but it works

        eq.code = response.code;
        await draw();
    } catch(error) {
        if(!(<string> error).startsWith("Empty error")) {
            console.warn(error);
            eq.writeError(error);
            return;
        }
    }
}

