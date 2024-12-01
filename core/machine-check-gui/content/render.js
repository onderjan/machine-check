const mainArea = document.getElementById("main_area");
const mainCanvas = document.getElementById("main_canvas");

var mainContext = mainCanvas.getContext("2d");

const devicePixelRatio = window.devicePixelRatio || 1;

function adjustForPixelRatio(v) {
    if (Array.isArray(v)) {
        return v.map((e, i) => Math.round(e * devicePixelRatio));
    }
    return Math.round(v * devicePixelRatio);
}

const tileSizePx = adjustForPixelRatio([30, 30]);
const tilePaddingPx = adjustForPixelRatio([16, 16]);
const tileDifferencePx = [tileSizePx[0] + tilePaddingPx[0], tileSizePx[1] + tilePaddingPx[1]];

const arrowLengthPx = adjustForPixelRatio(4);
const arrowWidthPx = adjustForPixelRatio(4);

const fontSizePx = adjustForPixelRatio(12);

var storedContent = null;

function fixResizedCanvas() {
    // fix for device pixel ratio
    var mainAreaRect = mainArea.getBoundingClientRect();

    mainCanvas.width = mainAreaRect.width * devicePixelRatio;
    mainCanvas.height = mainAreaRect.height * devicePixelRatio;
    mainCanvas.style.width = mainAreaRect.width + 'px'
    mainCanvas.style.height = mainAreaRect.height + 'px'

    mainContext.font = fontSizePx + "px sans-serif";

    // make sure we stroke true pixels
    mainContext.resetTransform();
    mainContext.translate(0.5, 0.5);
}

function onResize() {
    fixResizedCanvas();
    render();
}


window.addEventListener("resize", onResize);

fixResizedCanvas();


function drawPredecessorReference(node_id, middle) {
    mainContext.beginPath();
    mainContext.moveTo(middle[0] - tileSizePx[0] / 2, middle[1] - tileSizePx[0] / 3);
    mainContext.lineTo(middle[0] + tileSizePx[0] / 4, middle[1] - tileSizePx[0] / 3);
    mainContext.lineTo(middle[0] + tileSizePx[0] / 2, middle[1]);
    mainContext.lineTo(middle[0] + tileSizePx[0] / 4, middle[1] + tileSizePx[0] / 3);
    mainContext.lineTo(middle[0] - tileSizePx[0] / 2, middle[1] + tileSizePx[0] / 3);
    mainContext.lineTo(middle[0] - tileSizePx[0] / 2, middle[1] - tileSizePx[0] / 3);
    mainContext.stroke();
    mainContext.fillText(node_id, middle[0] - tileSizePx[0] / 16, middle[1]);
}

function drawSuccessorReference(node_id, middle) {
    mainContext.beginPath();
    mainContext.moveTo(middle[0] + tileSizePx[0] / 2, middle[1] - tileSizePx[0] / 3);
    mainContext.lineTo(middle[0] - tileSizePx[0] / 4, middle[1] - tileSizePx[0] / 3);
    mainContext.lineTo(middle[0] - tileSizePx[0] / 2, middle[1]);
    mainContext.lineTo(middle[0] - tileSizePx[0] / 4, middle[1] + tileSizePx[0] / 3);
    mainContext.lineTo(middle[0] + tileSizePx[0] / 2, middle[1] + tileSizePx[0] / 3);
    mainContext.lineTo(middle[0] + tileSizePx[0] / 2, middle[1] - tileSizePx[0] / 3);
    mainContext.stroke();
    mainContext.fillText(node_id, middle[0] + tileSizePx[0] / 16, middle[1]);
}

function render() {
    // clear the canvas
    mainContext.clearRect(-0.5, -0.5, mainCanvas.width - 0.5, mainCanvas.height - 0.5);

    // return if there is no content
    if (storedContent == null) {
        return;
    }
    const nodes = storedContent.state_space.nodes;

    // render the nodes at the computed tiles
    const basePx = tilePaddingPx;


    mainContext.textAlign = "center";
    mainContext.textBaseline = "middle";

    for (const node_id of Object.keys(nodes)) {
        const node = nodes[node_id];
        if (!node.render.tile) {
            // node not reachable
            continue;
        }

        // --- TILE ---

        const node_tile = node.render.tile;
        const startPx = node.render.tile.map((e, i) => basePx[i] + e * tileDifferencePx[i]);
        const middlePx = startPx.map((e, i) => e + tileSizePx[i] / 2);

        // render the node tile
        mainContext.beginPath();
        mainContext.rect(startPx[0], startPx[1], tileSizePx[0], tileSizePx[1]);
        mainContext.stroke();
        mainContext.fillText(node_id, middlePx[0], middlePx[1]);


        const outgoingPx = [startPx[0] + tileSizePx[0], startPx[1] + tileSizePx[1] / 2];
        const stagingStartPx = [startPx[0] + tileSizePx[0] + tilePaddingPx[0] / 2 - arrowLengthPx / 2, startPx[1] + tileSizePx[1] / 2];

        // --- PREDECESSORS ---
        var predecessor_y_position_add = 1;
        for (const predecessor_id of nodes[node_id].incoming) {
            // do not draw references for identity or canonical predecessors
            if (predecessor_id == node_id || predecessor_id == nodes[node_id].render.pred) {
                continue;
            }

            // draw a reference
            const referenceMiddlePx = [middlePx[0] - tileDifferencePx[0], middlePx[1] + predecessor_y_position_add * tileDifferencePx[1]];


            drawPredecessorReference(predecessor_id, referenceMiddlePx);

            // outward line
            mainContext.beginPath();
            mainContext.moveTo(referenceMiddlePx[0] + tileSizePx[0] / 2, referenceMiddlePx[1]);
            mainContext.lineTo(referenceMiddlePx[0] + tileDifferencePx[0] / 2 - arrowLengthPx / 2, referenceMiddlePx[1]);
            mainContext.stroke();


            predecessor_y_position_add += 1;
        }

        // draw a staging line if necessary
        if (predecessor_y_position_add != 1) {
            mainContext.beginPath();
            mainContext.moveTo(startPx[0] - tilePaddingPx[0] / 2 - arrowLengthPx / 2, middlePx[1]);
            mainContext.lineTo(startPx[0] - tilePaddingPx[0] / 2 - arrowLengthPx / 2, middlePx[1] + (predecessor_y_position_add - 1) * tileDifferencePx[1]);
            mainContext.stroke();
        }


        // --- SUCCESSORS ---

        // treat identity successors specially 
        var nonIdentitySuccessors = 0;
        for (const successor_id of nodes[node_id].outgoing) {
            if (successor_id != node_id) {
                nonIdentitySuccessors += 1;
            }
        }

        // render the end part of the arrow
        var y_position_add = 0;
        var previous_y_position_add = 0;
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

            const restagingPx = [stagingStartPx[0], stagingStartPx[1] + y_position_add * tileDifferencePx[1]];

            const successor = nodes[successor_id];

            var ingoingXPx = startPx[0] + tileSizePx[0] + tilePaddingPx[0];
            // adjust if this is the canonical occurence
            if (successor.render.pred == node_id) {
                ingoingXPx = basePx[0] + nodes[successor_id].render.tile[0] * tileDifferencePx[0];
            }

            const ingoingPx = [ingoingXPx, restagingPx[1]];


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
            if (successor.render.pred != node_id) {
                const referenceTile = [node_tile[0] + 1, node_tile[1] + y_position_add]
                const startPx = referenceTile.map((e, i) => basePx[i] + e * (tilePaddingPx[i] + tileSizePx[i]));
                const middlePx = startPx.map((e, i) => e + tileSizePx[i] / 2);

                drawSuccessorReference(successor_id, middlePx);
            }
            previous_y_position_add = y_position_add;
            y_position_add += nodes[successor_id].render.reserve;
        }

        // render the staging part of the arrow if there are at some successors
        if (nodes[node_id].outgoing.length) {
            const stagingEndPx = [stagingStartPx[0], stagingStartPx[1] + previous_y_position_add * tileDifferencePx[1]];
            mainContext.beginPath();
            mainContext.moveTo(outgoingPx[0], outgoingPx[1]);
            mainContext.lineTo(stagingStartPx[0], stagingStartPx[1]);
            mainContext.lineTo(stagingEndPx[0], stagingEndPx[1]);
            mainContext.stroke();
        }

    }

    document.getElementById("state_space").innerText = JSON.stringify(content);
}
