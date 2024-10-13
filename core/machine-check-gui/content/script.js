


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