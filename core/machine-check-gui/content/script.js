const mainCanvas = document.getElementById("main_canvas");

// fix for device pixel ratio
const devicePixelRatio = window.devicePixelRatio || 1
var mainCanvasRect = mainCanvas.getBoundingClientRect();
mainCanvas.width = mainCanvasRect.width * devicePixelRatio
mainCanvas.height = mainCanvasRect.height * devicePixelRatio
mainCanvas.style.width = mainCanvasRect.width + 'px'
mainCanvas.style.height = mainCanvasRect.height + 'px'


const mainContext = mainCanvas.getContext("2d");
//mainContext.scale(devicePixelRatio, devicePixelRatio)

// make sure we stroke true pixels
mainContext.translate(0.5, 0.5);

const tileSizePx = [30, 30];
const tilePaddingPx = [16, 16];
const tileDifferencePx = [tileSizePx[0] + tilePaddingPx[0], tileSizePx[1] + tilePaddingPx[1]];

const arrowLengthPx = 4;
const arrowWidthPx = 4;


async function callApi(action, method) {
    try {
        const response = await fetch("/api/" + action, {
            method,
        });
        if (!response.ok) {
            throw new Error("Non-OK response", response);
        }
        const responseContent = await response.json();
        console.log("Response received", response);
        return responseContent;
    } catch (error) {
        console.error(error.message);
        return null;
    }
}

async function showInitialContent() {
    console.log("Getting the initial content");
    const content = await callApi("content", "GET");
    render(content);
}

async function stepVerification() {
    console.log("Stepping verification");
    const content = await callApi("step_verification", "POST");
    render(content);
}

showInitialContent();