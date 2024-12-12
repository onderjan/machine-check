function update(content) {

    console.log("Rendering", JSON.stringify(content));
    if (content == null) {
        return;
    }
    storedContent = content;

    const nodes = storedContent.state_space.nodes;

    var topologicalIncomingDegree = {};
    var topologicalOutgoing = {};

    // toss out the previous rendering data
    for (node_id of Object.keys(nodes)) {
        console.log(nodes);
        console.log(nodes[node_id], "node", node_id, nodes[node_id]);
        nodes[node_id].internal = {};
        topologicalIncomingDegree[node_id] = 0;
        topologicalOutgoing[node_id] = [];

    }


    //mainContext.strokeStyle = "black";
    //mainContext.fillStyle = "black";

    // find the leaves in the visualisation directed acyclic graph

    var queue = ["0"];
    var visited = new Set();
    while (queue.length) {
        const node_id = queue.shift();
        visited.add(node_id);

        for (const successor_id of nodes[node_id].outgoing) {

            if (!visited.has(successor_id)) {
                topologicalOutgoing[node_id].push(successor_id);
                topologicalIncomingDegree[successor_id] += 1;

                nodes[successor_id].internal = {
                    pred: node_id,
                };
                queue.push(successor_id);
            }
        }
    }

    console.log("Topological outgoing", topologicalOutgoing);

    // create the topological sort using Kahn's algorithm
    // so that there is no recursion

    queue = ["0"];
    sorted = [];

    while (queue.length) {
        const node_id = queue.shift();
        sorted.push(node_id);

        for (const successor_id of topologicalOutgoing[node_id]) {
            // only queue the successor if has no other topological edges
            console.log("going from", node_id, "to", successor_id, "topological incoming degree", topologicalIncomingDegree[successor_id]);
            topologicalIncomingDegree[successor_id] -= 1;
            if (topologicalIncomingDegree[successor_id] == 0) {
                queue.push(successor_id);
            }
        }
    }

    console.log("Topologically sorted", sorted);

    // compute the number of reserved tile y-positions for each node
    // using reverse topological sort
    for (let index = sorted.length - 1; index >= 0; index--) {
        const node_id = sorted[index];

        var predecessor_reserve = 0;
        var successor_reserve = 0;

        for (const predecessor_id of nodes[node_id].incoming) {
            if (predecessor_id != node_id) {
                // reserve one position for each non-identity predecessor
                predecessor_reserve += 1;
            }
        }

        for (const successor_id of nodes[node_id].outgoing) {
            if (successor_id != node_id) {
                // reserve the y-positions of each non-identity successor
                // but reserve only one if they do not consider this a canonical precedessor
                console.log("Node/successor", node_id, nodes[node_id], successor_id, nodes[successor_id]);
                if (nodes[successor_id].internal.pred == node_id) {
                    successor_reserve += nodes[successor_id].internal.reserve;
                } else {
                    successor_reserve += 1;
                }
            }
        }

        console.log("Reserves for node", node_id, predecessor_reserve, successor_reserve);

        // reserve the maximal one but at least one y-position
        nodes[node_id].internal.reserve = Math.max(predecessor_reserve, successor_reserve, 1);

        for (const predecessor_id of nodes[node_id].incoming) {
            if (!visited.has(predecessor_id)) {
                // breadth first search here to 
                queue.push(predecessor_id);
            }
        }
    }

    // we now have everything topologically sorted
    // stage tile positions according to the reverse topological sort

    // stage tile positions by depth-first search, taking the reserved y-positions into account
    queue = ["0"];
    visited = new Set();
    nodes["0"].internal.tile = [0, 0];

    while (queue.length) {
        const node_id = queue.shift();
        visited.add(node_id);

        const node_tile = nodes[node_id].internal.tile;

        var numNonidentityCanonicalSuccessors = 0;
        for (const successor_id of nodes[node_id].outgoing) {
            if (successor_id != node_id && nodes[successor_id].internal.pred == successor_id) {
                numNonidentityCanonicalSuccessors += 1;
            }
        }

        var y_position_add = 0;
        for (const successor_id of nodes[node_id].outgoing) {
            if (successor_id == node_id) {
                // trivial successor, do not stage a tile
                continue;
            }

            if (!visited.has(successor_id)) {
                // offset the tile if it has a non-identity non-canonical predecessor
                // and there is more than one non-identity canonical successor of the original node
                // (otherwise, the staging lines of incoming references to the successor
                // and outgoing transitions from the predecessor would overlap)

                var offset = 0;
                if (numNonidentityCanonicalSuccessors > 1) {
                    for (const sibling_id of nodes[successor_id].incoming) {
                        if (sibling_id != node_id && sibling_id != successor_id) {
                            offset = 1;
                        }
                    }
                }

                if (nodes[successor_id].internal.pred == node_id) {
                    nodes[successor_id].internal.tile = [node_tile[0] + 1 + offset, node_tile[1] + y_position_add];
                    queue.push(successor_id);
                }
            }
            if (nodes[successor_id].internal.pred == node_id) {
                y_position_add += nodes[successor_id].internal.reserve;
            } else {
                y_position_add += 1;

            }
        }
    }

    // map tiles to nodes
    storedContent.internal = { tileMap: {} };

    for (node_id of Object.keys(nodes)) {
        const tile = nodes[node_id].internal.tile;
        console.log("Tile", tile);

        if (!storedContent.internal.tileMap[tile[0]]) {
            storedContent.internal.tileMap[tile[0]] = {};
        }

        storedContent.internal.tileMap[tile[0]][tile[1]] = node_id;
    }

    console.log("Tile map", storedContent.internal.tileMap);

    document.getElementById("state_space").innerText = JSON.stringify(storedContent);

    // render the updated content
    render();
}

function handleClick(event) {
    event.preventDefault();


    if (storedContent == null) {
        return;
    }

    const adjustedPx = adjustForPixelRatio([event.offsetX, event.offsetY]);

    const tile = canvasPxToTile(adjustedPx);

    const startPx = tile.map((e, i) => currentOffsetPx[i] + e * tileDifferencePx[i]);
    const middlePx = startPx.map((e, i) => e + tileSizePx[i] / 2);

    // render the node tile
    mainContext.beginPath();
    mainContext.rect(startPx[0], startPx[1], tileSizePx[0], tileSizePx[1]);
    mainContext.stroke();

    const node_column = storedContent.internal.tileMap[tile[0]];
    if (!node_column) {
        selectedNodeId = null;
        render();
        return;
    }
    const node_id = node_column[tile[1]];
    selectedNodeId = node_id;

    render();
}

var translationStartMouse = null;
var translationStartOffsetPx = null;

function handleMouseDown(event) {
    event.preventDefault();
    console.log("Mouse down", event);

    // change the viewpoint translation if using the auxiliary (middle) button
    if (event.button != 1) {
        return;
    }

    translationStartMouse = [event.offsetX, event.offsetY];
    translationStartOffsetPx = currentOffsetPx;
    console.log("Translation start", translationStart);
}

function handleMouseMove(event) {
    event.preventDefault();

    if (translationStartMouse == null) {
        return;
    }

    const offsetTranslation = [event.offsetX - translationStartMouse[0], event.offsetY - translationStartMouse[1]];
    const adjustedOffsetTranslation = adjustForPixelRatio(offsetTranslation);

    // apply translation
    currentOffsetPx = translationStartOffsetPx.map((e, i) => e + adjustedOffsetTranslation[i]);

    // render
    render();
}

function handleMouseUp(event) {
    event.preventDefault();

    // change the viewpoint translation if using the auxiliary (middle) button
    if (event.button != 1) {
        return;
    }

    if (translationStartMouse == null) {
        return;
    }
    console.log("Applying translation");
    console.log("Before: " + translationStartOffsetPx);

    const offsetTranslation = [event.offsetX - translationStartMouse[0], event.offsetY - translationStartMouse[1]];
    const adjustedOffsetTranslation = adjustForPixelRatio(offsetTranslation);

    // apply translation
    currentOffsetPx = translationStartOffsetPx.map((e, i) => e + adjustedOffsetTranslation[i]);
    translationStartMouse = null;
    translationStartOffsetPx = null;

    console.log("After: " + currentOffsetPx);
    // render
    render();
}


function handleMouseOut(event) {
    // cancel any translation
    translationStart = null;
}