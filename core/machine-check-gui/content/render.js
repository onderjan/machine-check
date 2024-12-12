const mainArea = document.getElementById("main_area");
const mainCanvas = document.getElementById("main_canvas");
const execName = document.getElementById("exec_name");
const stateFields = document.getElementById("state_fields");

var mainContext = mainCanvas.getContext("2d");

const devicePixelRatio = window.devicePixelRatio || 1;

function adjustForPixelRatio(v) {
    if (Array.isArray(v)) {
        return v.map((e, i) => Math.round(e * devicePixelRatio));
    }
    return Math.round(v * devicePixelRatio);
}

function adjustFromPixelRatio(v) {
    if (Array.isArray(v)) {
        return v.map((e, i) => Math.round(e / devicePixelRatio));
    }
    return Math.round(v / devicePixelRatio);
}


const tileSizePx = adjustForPixelRatio([30, 30]);
const tilePaddingPx = adjustForPixelRatio([16, 16]);
const tileDifferencePx = [tileSizePx[0] + tilePaddingPx[0], tileSizePx[1] + tilePaddingPx[1]];

const arrowLengthPx = adjustForPixelRatio(4);
const arrowWidthPx = adjustForPixelRatio(4);

const fontSizePx = adjustForPixelRatio(12);

var storedContent = null;
var currentOffsetPx = tilePaddingPx;
var selectedNodeId = null;

function canvasPxToTile(coords) {
    return [Math.floor((coords[0] - currentOffsetPx[0] + tilePaddingPx[0] / 2) / tileDifferencePx[0]),
    Math.floor((coords[1] - currentOffsetPx[1] + tilePaddingPx[1] / 2) / tileDifferencePx[1])];
}

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
    // clear the state fields
    const elements = stateFields.getElementsByClassName("dynamic");
    while (elements[0]) {
        elements[0].parentNode.removeChild(elements[0]);
    }

    // clear the canvas
    mainContext.clearRect(-0.5, -0.5, mainCanvas.width - 0.5, mainCanvas.height - 0.5);

    // return if there is no content
    if (storedContent == null) {
        return;
    }

    execName.innerText = storedContent.exec_name;

    renderCanvas();
    renderStateFields();

}


function renderCanvas() {
    const nodes = storedContent.state_space.nodes;

    // render the nodes at the computed tiles
    const basePx = currentOffsetPx;

    mainContext.textAlign = "center";
    mainContext.textBaseline = "middle";

    for (const node_id of Object.keys(nodes)) {
        const node = nodes[node_id];
        if (!node.internal.tile) {
            // node not reachable
            continue;
        }

        // --- TILE ---

        const node_tile = node.internal.tile;
        const startPx = node.internal.tile.map((e, i) => basePx[i] + e * tileDifferencePx[i]);
        const middlePx = startPx.map((e, i) => e + tileSizePx[i] / 2);

        if (node_id == selectedNodeId) {
            mainContext.fillStyle = "lightblue";
        } else {
            mainContext.fillStyle = "white";

        }

        // render the node tile
        mainContext.beginPath();
        mainContext.rect(startPx[0], startPx[1], tileSizePx[0], tileSizePx[1]);
        mainContext.fill();
        mainContext.stroke();

        mainContext.fillStyle = "black";
        mainContext.fillText(node_id, middlePx[0], middlePx[1]);


        const outgoingPx = [startPx[0] + tileSizePx[0], startPx[1] + tileSizePx[1] / 2];
        const stagingStartPx = [startPx[0] + tileSizePx[0] + tilePaddingPx[0] / 2 - arrowLengthPx / 2, startPx[1] + tileSizePx[1] / 2];

        // --- PREDECESSORS ---
        var predecessor_y_position_add = 1;
        for (const predecessor_id of nodes[node_id].incoming) {
            // do not draw references for identity or canonical predecessors
            if (predecessor_id == node_id || predecessor_id == nodes[node_id].internal.pred) {
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
            if (successor.internal.pred == node_id) {
                ingoingXPx = basePx[0] + nodes[successor_id].internal.tile[0] * tileDifferencePx[0];
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
            if (successor.internal.pred != node_id) {
                const referenceTile = [node_tile[0] + 1, node_tile[1] + y_position_add]
                const startPx = referenceTile.map((e, i) => basePx[i] + e * (tilePaddingPx[i] + tileSizePx[i]));
                const middlePx = startPx.map((e, i) => e + tileSizePx[i] / 2);

                drawSuccessorReference(successor_id, middlePx);
            }
            previous_y_position_add = y_position_add;
            if (successor.internal.pred == node_id) {
                y_position_add += nodes[successor_id].internal.reserve;
            } else {
                y_position_add += 1;
            }
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
}

function addStateFieldsRow(field, value, fieldClasses, valueClasses) {
    if (value == null) {
        value = "(display error)";
        console.error("Value null for field", field);
    }

    const row = stateFields.insertRow();
    row.classList.add("dynamic");
    const fieldCell = row.insertCell(0);
    fieldCell.innerText = field;
    const valueCell = row.insertCell(1);
    valueCell.innerText = value;
    if (fieldClasses != null) {
        for (cls of fieldClasses) {
            fieldCell.classList.add(cls);
        }
    }
    if (valueClasses != null) {
        for (cls of valueClasses) {
            valueCell.classList.add(cls);
        }
    }
}

function addAuxiliaryStateFieldsRow(text) {
    const row = stateFields.insertRow();
    row.classList.add("dynamic");
    const fieldCell = row.insertCell(0);
    fieldCell.innerText = text;
    fieldCell.colSpan = 2;
}

function renderStateFields() {
    const nodes = storedContent.state_space.nodes;

    var selectedNode = null;
    if (selectedNodeId != null) {
        selectedNode = nodes[selectedNodeId];
    }
    if (selectedNode == null) {
        addAuxiliaryStateFieldsRow("(no node selected)");
        return;
    }

    addStateFieldsRow("id", selectedNodeId, ["italic", "bold"], null);

    if (selectedNode.panic != null) {
        var value = null;
        const zero = selectedNode.panic.zero;
        const one = selectedNode.panic.one;
        if (one == true) {
            if (zero == true) {
                value = "unknown";
            } else {
                value = "true";
            }
        } else {
            if (zero == true) {
                value = "false";
            } else {
                // should never occur
            }
        }

        addStateFieldsRow("panic", value, ["italic", "bold"], ["bold"]);
    }

    console.log("Selected node", selectedNode, selectedNode.fields.size);
    if (Object.keys(selectedNode.fields).length == 0 && selectedNode.panic == null) {
        if (selectedNodeId == 0) {
            addAuxiliaryStateFieldsRow("(inital state has no fields)");
        } else {
            addAuxiliaryStateFieldsRow("(state has no fields)");
        }
        return;
    }

    for (const fieldName in selectedNode.fields) {
        const field = selectedNode.fields[fieldName];
        var value = null;

        if (Object.keys(field.domains).length == 1 && field.domains["tv"] != null) {
            value = "ABC";
            if (field.type == "bitvector") {
                const bitWidth = field.bit_width;
                value = "";
                const tv = field.domains["tv"];
                var ones = tv.ones;
                var zeros = tv.zeros;
                for (var i = 0; i < bitWidth; ++i) {
                    var bitValue;
                    if (ones % 2 == 1) {
                        if (zeros % 2 == 1) {
                            bitValue = 'X';
                        } else {
                            bitValue = '1';
                        }
                    } else {
                        if (zeros % 2 == 1) {
                            bitValue = '0';
                        } else {
                            // should never occur
                            bitValue = null;
                        }

                    }
                    if (bitValue != null) {
                        value = bitValue + value;
                    } else {
                        value = null;
                    }
                    ones = Math.floor(ones / 2);
                    zeros = Math.floor(zeros / 2);
                }
                if (value != null) {
                    value = "\"" + value + "\"";
                }
            }
        }

        addStateFieldsRow(fieldName, value, null, ["monospace"]);
    }
}