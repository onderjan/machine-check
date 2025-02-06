mod render;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use bimap::BiHashMap;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys::Array, Request, RequestInit, RequestMode, Response};

use super::view::Content;

pub enum Action {
    GetContent,
    Step,
}

pub async fn update(action: Action) {
    let result = call_backend(action).await;
    let json = match result {
        Ok(ok) => ok,
        Err(err) => panic!("{:?}", err),
    };
    let content: Content =
        serde_wasm_bindgen::from_value(json).expect("Content should be convertible from JSON");

    let cons = Array::new_with_length(1);
    cons.set(0, JsValue::from_str(&format!("{:?}", content)));
    web_sys::console::log(&cons);

    let view = View::new(content);

    render::render(&view);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Tile {
    x: u64,
    y: u64,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum TileType {
    Node(String),
    IncomingReference(String),
    OutgoingReference(String),
}

struct View {
    content: Content,
    tiling: BiHashMap<Tile, TileType>,
    split_lengths: HashMap<String, u64>,
}

impl View {
    fn new(content: Content) -> View {
        // compute predecessor/successor reserved y-positions using topological sorting
        let sorted = topological_sort(&content);
        let mut reserved = HashMap::<String, (usize, usize)>::new();

        // TODO: implement the better algorithm
        for (node_id, node) in &content.state_space.nodes {
            reserved.insert(node_id.clone(), (node.incoming.len(), node.outgoing.len()));
        }

        // stage tile positions by depth-first search, taking the reserved y-positions into account
        let mut tiling = BiHashMap::new();
        let mut split_lengths = HashMap::new();
        tiling.insert(Tile { x: 0, y: 0 }, TileType::Node(String::from("0")));
        let mut stack = Vec::new();
        stack.push(String::from("0"));

        while let Some(node_id) = stack.pop() {
            let node_tile = *tiling
                .get_by_right(&TileType::Node(node_id.clone()))
                .expect("Node should be in tiling");

            let mut y_add = 0;

            for successor_id in content
                .state_space
                .nodes
                .get(&node_id)
                .unwrap()
                .outgoing
                .iter()
            {
                if !tiling.contains_right(&TileType::Node(successor_id.clone())) {
                    tiling.insert(
                        Tile {
                            x: node_tile.x + 1,
                            y: node_tile.y + y_add,
                        },
                        TileType::Node(successor_id.clone()),
                    );
                    stack.push(successor_id.clone());
                } else {
                    tiling.insert(
                        Tile {
                            x: node_tile.x + 1,
                            y: node_tile.y + y_add,
                        },
                        TileType::OutgoingReference(successor_id.clone()),
                    );
                }

                y_add += 1;
            }
            split_lengths.insert(node_id, y_add.saturating_sub(1));
        }

        let cons = Array::new_with_length(2);
        cons.set(0, JsValue::from_str("Tiling"));
        cons.set(1, JsValue::from_str(&format!("{:?}", tiling)));
        web_sys::console::log(&cons);

        View {
            content,
            tiling,
            split_lengths,
        }
    }
}

pub async fn call_backend(action: Action) -> Result<JsValue, JsValue> {
    let (method, url_action) = match action {
        Action::GetContent => ("GET", "content"),
        Action::Step => ("POST", "step_verification"),
    };

    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);

    let url = format!("/api/{}", url_action);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into().unwrap();

    let json = JsFuture::from(resp.json()?).await?;

    Ok(json)
}

fn topological_sort(content: &Content) -> Vec<String> {
    // construct a topological ordering using Kahn's algorithm on a DAG
    // the node without any incoming edge is the root
    let (mut dag_outgoing, mut dag_incoming_degree) = {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(String::from("0"));

        // construct Directed Acyclic Graph

        let mut dag_outgoing: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut dag_incoming_degree: BTreeMap<String, usize> = BTreeMap::new();

        while let Some(node_id) = queue.pop_front() {
            visited.insert(node_id.clone());

            for successor_id in &content.state_space.nodes.get(&node_id).unwrap().outgoing {
                if !visited.contains(successor_id) {
                    dag_outgoing
                        .entry(node_id.clone())
                        .or_default()
                        .insert(successor_id.clone());
                    *dag_incoming_degree.entry(successor_id.clone()).or_default() += 1;

                    queue.push_back(successor_id.clone());
                }
            }
        }
        (dag_outgoing, dag_incoming_degree)
    };

    // use Kahn's algorithn

    let mut queue = VecDeque::new();
    queue.push_back(String::from("0"));
    let mut sorted = Vec::new();

    while let Some(node_id) = queue.pop_front() {
        sorted.push(node_id.clone());

        for successor_id in dag_outgoing.entry(node_id).or_default().iter() {
            let incoming = dag_incoming_degree.entry(successor_id.clone()).or_default();

            assert_ne!(*incoming, 0);
            *incoming -= 1;
            if *incoming == 0 {
                queue.push_back(successor_id.clone());
            }
        }
    }
    let cons = Array::new_with_length(2);
    cons.set(0, JsValue::from_str("Topologically sorted"));
    cons.set(1, JsValue::from_str(&format!("{:?}", sorted)));
    web_sys::console::log(&cons);

    sorted
}
