import { returnHome } from "./background";
import { EditAction, EditPayload, EquationBox, expressions } from "./equations";
import { listen } from "@tauri-apps/api/event";
import { draw } from "./renderer";
import { invoke } from "@tauri-apps/api";
import { variableBoxes } from "./variables";
import { functionSet } from "./functions";

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

interface Response {
    code: string,
    num?: number,
}

await listen('edited', async event => {
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
        variables = eq?.getVariables();
        fnName = eq.functionCharacter(); 

        if(fnName) {
            functionSet.add(fnName);
            
            if([...variables].some(e => functionSet.has(e)))
                throw Error("A variable can't be called like a function");
        }
    } catch (error) {
        console.warn(error);
        eq.writeError(error);
        return;
    }

    const undefinedVariables = [...variables].filter(e => !variableBoxes.has(e))
        .filter(e => !"xye".includes(e));
    eq.showUndefinedVariables(undefinedVariables);
    if(undefinedVariables.length > 0) {
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
        console.warn(error);
        eq.writeError(error);
        return;
    }

    eq.toggleError();
});