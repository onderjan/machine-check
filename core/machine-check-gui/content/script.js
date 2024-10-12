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

async function render(content) {
    console.log("Rendering", JSON.stringify(content));
    if (content === null) {
        return;
    }
    const nodes = content.state_space.nodes;

    // toss out the previous rendering data
    for (node_id in Object.values(nodes)) {
        nodes[node_id].render = {};
    }


    //mainContext.strokeStyle = "black";
    //mainContext.fillStyle = "black";

    nodes["0"].render = { tile: [0, 0] };


    // assign tile positions by depth-first search

    const queue = ["0"];
    const visited = new Set();
    while (queue.length) {
        const node_id = queue.shift();
        const node_tile = nodes[node_id].render.tile;

        visited.add(node_id);

        var i = 0;
        for (const successor_id of nodes[node_id].outgoing) {
            if (successor_id == node_id) {
                // trivial successor, do not stage a tile
                continue;
            }

            if (!visited.has(successor_id)) {
                nodes[successor_id].render = {
                    pred: node_id,
                    tile: [node_tile[0] + 1, node_tile[1] + i]
                };
                queue.push(successor_id);
            }
            i += 1;
        }
    }

    // clear the canvas
    mainContext.clearRect(-0.5, -0.5, mainCanvas.width - 0.5, mainCanvas.height - 0.5);

    // render the nodes at the computed tiles

    const tileSizePx = [30, 30];
    const tilePaddingPx = [16, 16];
    const tileDifferencePx = [tileSizePx[0] + tilePaddingPx[0], tileSizePx[1] + tilePaddingPx[1]];

    const basePx = tilePaddingPx;

    const arrowLengthPx = 4;
    const arrowWidthPx = 4;

    mainContext.textAlign = "center";
    mainContext.textBaseline = "middle";

    for (const node_id of Object.keys(nodes)) {
        const node = nodes[node_id];
        if (!node.render.tile) {
            // node not reachable
            continue;
        }

        const node_tile = node.render.tile;
        const startPx = node.render.tile.map((e, i) => basePx[i] + e * tileDifferencePx[i]);
        const middlePx = startPx.map((e, i) => e + tileSizePx[i] / 2);
        console.log(node, "rect", startPx, tileSizePx);

        // render the node tile
        mainContext.beginPath();
        mainContext.rect(startPx[0], startPx[1], tileSizePx[0], tileSizePx[1]);
        mainContext.stroke();
        mainContext.fillText(node_id, middlePx[0], middlePx[1]);


        const outgoingPx = [startPx[0] + tileSizePx[0], startPx[1] + tileSizePx[1] / 2];
        const stagingStartPx = [startPx[0] + tileSizePx[0] + tilePaddingPx[0] / 2, startPx[1] + tileSizePx[1] / 2];

        // treat identity successors specially 
        var nonIdentitySuccessors = 0;
        for (const successor_id of nodes[node_id].outgoing) {
            if (successor_id != node_id) {
                nonIdentitySuccessors += 1;
            }
        }

        // render the staging part of the arrow if there are at some successors
        if (nodes[node_id].outgoing.length) {
            var numStagedTiles = 0;
            if (nonIdentitySuccessors) {
                numStagedTiles = nonIdentitySuccessors - 1;
            }

            const stagingEndPx = [stagingStartPx[0], stagingStartPx[1] + numStagedTiles * tileDifferencePx[1]];
            mainContext.beginPath();
            mainContext.moveTo(outgoingPx[0], outgoingPx[1]);
            mainContext.lineTo(stagingStartPx[0], stagingStartPx[1]);
            mainContext.lineTo(stagingEndPx[0], stagingEndPx[1]);
            mainContext.stroke();
        }


        // render the end part of the arrow
        var i = 0;
        for (const successor_id of nodes[node_id].outgoing) {
            if (successor_id == node_id) {
                // identity successor, render as a loop
                const loopStartPx = [stagingStartPx[0], startPx[1] - tilePaddingPx[1] / 2];
                const loopMiddlePx = [middlePx[0], loopStartPx[1]];
                const loopEndPx = [loopMiddlePx[0], startPx[1]];

                // draw the lines
                mainContext.beginPath();
                mainContext.moveTo(stagingStartPx[0], stagingStartPx[1]);
                mainContext.lineTo(loopStartPx[0], loopStartPx[1]);
                mainContext.lineTo(loopMiddlePx[0], loopMiddlePx[1]);
                mainContext.lineTo(loopEndPx[0], loopEndPx[1]);
                mainContext.stroke();

                // draw the arrowhead
                mainContext.beginPath();
                mainContext.lineTo(loopEndPx[0] - arrowWidthPx / 2, loopEndPx[1] - arrowLengthPx);
                mainContext.lineTo(loopEndPx[0] + arrowWidthPx / 2, loopEndPx[1] - arrowLengthPx);
                mainContext.lineTo(loopEndPx[0], loopEndPx[1]);
                mainContext.fill();

                continue;
            }

            const restagingPx = [stagingStartPx[0], stagingStartPx[1] + i * tileDifferencePx[1]];
            const ingoingPx = [startPx[0] + tileSizePx[0] + tilePaddingPx[0], restagingPx[1]];


            // draw the line
            mainContext.beginPath();
            mainContext.moveTo(restagingPx[0], restagingPx[1]);
            mainContext.lineTo(ingoingPx[0], ingoingPx[1]);
            mainContext.stroke();

            // draw the arrowhead
            mainContext.beginPath();
            mainContext.lineTo(ingoingPx[0] - arrowLengthPx, ingoingPx[1] - arrowWidthPx / 2);
            mainContext.lineTo(ingoingPx[0] - arrowLengthPx, ingoingPx[1] + arrowWidthPx / 2);
            mainContext.lineTo(ingoingPx[0], ingoingPx[1]);
            mainContext.fill();

            // if this is not the canonical occurence of the successor, render a reference
            const successor = nodes[successor_id];
            if (successor.render.pred != node_id) {
                const referenceTile = [node_tile[0] + 1, node_tile[1] + i]
                const startPx = referenceTile.map((e, i) => basePx[i] + e * (tilePaddingPx[i] + tileSizePx[i]));
                const middlePx = startPx.map((e, i) => e + tileSizePx[i] / 2);

                mainContext.beginPath();
                mainContext.ellipse(middlePx[0], middlePx[1], tileSizePx[0] / 2, tileSizePx[1] / 2, 0, 0, 2 * Math.PI);
                mainContext.stroke();
                mainContext.fillText(successor_id, middlePx[0], middlePx[1]);
            }

            i += 1;
        }
    }

    document.getElementById("state_space").innerText = JSON.stringify(content);
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