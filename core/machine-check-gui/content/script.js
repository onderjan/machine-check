
async function callApi(action, method) {
    try {
        const response = await fetch("/api/" + action, {
            method,
        });
        if (!response.ok) {
            throw new Error("Non-OK response", response);
        }
        const content = await response.json();
        console.log("Content received", response, content);

        document.getElementById("state_space").innerText = JSON.stringify(content);
    } catch (error) {
        console.error(error.message);
    }
}

async function showInitialContent() {
    console.log("Getting the initial content");
    callApi("content", "GET");
}

async function stepVerification() {
    console.log("Stepping verification");
    callApi("step_verification", "POST");
}

showInitialContent();