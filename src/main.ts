import { returnHome } from "./background";
import { CHANGED_EMIT_CODE, EditAction, EditPayload, EquationBox, expressions } from "./equations";
import { listen } from "@tauri-apps/api/event";
import { draw } from "./renderer";
import { invoke } from "@tauri-apps/api";
import { addFunction, functionSet } from "./functions";

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
    let fnName: string | null;
    try {
        fnName = eq.functionCharacter(); 
        variables = eq?.getVariables();

        if(fnName) {
            if(functionSet.has(fnName) && functionSet.get(fnName) !== id)
                throw Error("There's already a function with this name");

            if([...variables].some(e => e == fnName))
                throw Error("A variable can't be called like a function");
            
            variables.delete(fnName);
            eq.showUndefinedVariables(variables)

            functionSet.set(fnName, id);
            addFunction(fnName, latex, eq, payload.action);
            eq.toggleError();
            return;
        }
    } catch (error) {
        console.warn(error);
        eq.writeError(error);
        return;
    }

    
    if(eq.showUndefinedVariables(variables) > 0) {
        eq.toggleError();
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