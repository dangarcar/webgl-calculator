import { returnHome } from "./background";
import { CHANGED_EMIT_CODE, EditAction, EditPayload, EquationBox, expressions, functionSet, variableSet } from "./equations";
import { listen } from "@tauri-apps/api/event";
import { draw } from "./renderer";
import { invoke } from "@tauri-apps/api";

const moreBtn = document.getElementById("more");
moreBtn?.addEventListener("click", () => {
    const eq = EquationBox.createNew();
    const sidebar = document.getElementById("sidebar");
    sidebar?.appendChild(eq.htmlElement);
    eq.focus();
    draw();
});

const homeBtn = document.getElementById('home');
homeBtn?.addEventListener('click', returnHome);

window.addEventListener('DOMContentLoaded', () => {
    draw();
})

export interface Response {
    code: string,
    num?: number,
}

await listen(CHANGED_EMIT_CODE, async event => {
    const payload = <EditPayload> event.payload;
    const id = payload.id;
    const eq = expressions.get(id);
    const latex = payload.latex;

    if(!eq) throw Error("There isn't any equations to edit");
    eq.error = false; //Supose there aren't any errors right now, we'll discover them later

    if(payload.action == EditAction.ADD)
        eq.writeFunctionBrackets();

    let variables: Set<string>;
    try {
        const varName = eq.variableCharacter();
        const fnName = eq.functionCharacter(); 
        variables = eq.getVariables();

        if(eq.showUndefinedVariables(variables) > 0) {
            eq.toggleError();
            return;
        }

        if(varName) {
            eq.hideSolutionBox();
            if(variableSet.has(varName) && variableSet.get(varName) !== id)
                throw Error("There's already a function with this name");

            eq.setDrawable(false);

            variableSet.set(varName, id);
            const val = await addVariable(varName, eq, latex.substring(2), payload.action);
            eq.setSolutionValue(val!);
            eq.toggleError();
            return;
        } else {
            eq.setDrawable(true);
        }

        if(fnName) {
            if(functionSet.has(fnName) && functionSet.get(fnName) !== id)
                throw Error("There's already a function with this name");

            if([...variables].some(e => e == fnName))
                throw Error("A variable can't be called like a function");
            
            variables.delete(fnName);
            eq.showUndefinedVariables(variables)

            functionSet.set(fnName, id);
            await addFunction(fnName, latex, eq, payload.action);
            eq.toggleError();
            return;
        }
    } catch (error) {
        console.warn(error);
        eq.writeError(error);
        return;
    }

    try {
        const response = <Response> await invoke("process", { eq: latex });

        if(response.num !== null && response.num !== undefined) {
            eq.setSolutionValue(response.num);
        } else {
            eq.hideSolutionBox();

            eq.code = response.code;
            console.time();
            await draw();
            console.timeEnd();
        }
    } catch(error) {
        if(!(<string> error).startsWith("Empty error")) {
            console.warn(error);
            eq.writeError(error);
            return;
        }
    }

    eq.toggleError();
});

const addFunction = async (fnName: string, latex: string, eq: EquationBox, action: EditAction) => {
    const code = latex.substring(latex.indexOf('=')+1);
    const unknown = Math.min(latex.indexOf('x')>0? latex.indexOf('x'):1e9, latex.indexOf('y')>0? latex.indexOf('y'):1e9);

    try {
        fnName += latex[unknown]!;
        const response = <Response> await invoke('add_function', { name: fnName, content: code });

        if(action != EditAction.REFRESH)
            for(let id of expressions.keys())
                if(id != eq.number)
                    expressions.get(id)?.refresh();

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

const addVariable = async (varName: string, eq: EquationBox, latex: string, action: EditAction) => {
    const vars = eq.getVariables();
    if(vars.has('x') || vars.has('y')) {
        eq.writeError(new Error("A variable can't have x nor y because it has to be constant"));
        return;
    }
    
    try {
        const val = <number> await invoke('add_variable', { name: varName, content: latex });
        if(action != EditAction.REFRESH)
            expressions.forEach(e => e.refresh());
        
        return val;
    } catch(error) {
        console.warn(error);
        eq.writeError(error);
        return;
    }
}