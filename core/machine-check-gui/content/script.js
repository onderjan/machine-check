
async function showInitialContent() {
    console.log("Trying to get the initial content")

    try {
        const response = await fetch("/api/content");
        if (!response.ok) {
            throw new Error("Non-OK response", response);
        }
        const content = await response.json();
        console.log("Initial content received", response, content);

        document.getElementById("state_space").innerText = JSON.stringify(content);
    } catch (error) {
        console.error(error.message);
    }
}

showInitialContent();