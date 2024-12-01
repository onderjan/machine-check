function update(content) {

    console.log("Rendering", JSON.stringify(content));
    if (content == null) {
        return;
    }
    storedContent = content;

    const nodes = content.state_space.nodes;

    var topologicalIncomingDegree = {};
    var topologicalOutgoing = {};

    // toss out the previous rendering data
    for (node_id in Object.values(nodes)) {
        nodes[node_id].render = {};
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

                nodes[successor_id].render = {
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
                if (nodes[successor_id].render.pred == node_id) {
                    successor_reserve += nodes[successor_id].render.reserve;
                } else {
                    successor_reserve += 1;
                }
            }
        }

        console.log("Reserves for node", node_id, predecessor_reserve, successor_reserve);

        // reserve the maximal one but at least one y-position
        nodes[node_id].render.reserve = Math.max(predecessor_reserve, successor_reserve, 1);

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
    nodes["0"].render.tile = [0, 0];

    while (queue.length) {
        const node_id = queue.shift();
        visited.add(node_id);

        const node_tile = nodes[node_id].render.tile;

        var numNonidentityCanonicalSuccessors = 0;
        for (const successor_id of nodes[node_id].outgoing) {
            if (successor_id != node_id && nodes[successor_id].render.pred == successor_id) {
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

                nodes[successor_id].render.tile = [node_tile[0] + 1 + offset, node_tile[1] + y_position_add];
                queue.push(successor_id);
            }
            y_position_add += nodes[successor_id].render.reserve;
        }
    }

    // render the updated content
    render();
}