import { Graph } from "@viz-js/viz";
import { RegexEngine } from "regex-potata";

function graphFromRegex(regex: RegexEngine) {
  const states = regex.nfaStates();
  const endState = states.length - 1;

  const config: Graph = {
    graphAttributes: {
      bgcolor: "none",
      rankdir: "LR",
    },
    nodeAttributes: {
      shape: "circle",
      color: "white",
      penwidth: 2,
      fontcolor: "white",
      fontname: "Arial",
    },
    edgeAttributes: {
      color: "#67e8f9",
      fontcolor: "white",
      fontname: "Arial",
    },
    nodes: [
      { name: "", attributes: { shape: "none" } },
      {
        name: endState.toString(),
        attributes: { shape: "doublecircle", color: "#67e8f9" },
      },
    ],
    edges: [{ tail: "", head: "0" }],
    subgraphs: [],
  };

  for (const state of states) {
    const transitions = regex.nfaTransition(state);

    if (transitions) {
      for (const transition of transitions) {
        config.edges!.push({
          tail: state.toString(),
          head: transition.end.toString(),
          attributes: { label: transition.toString() },
        });
      }
    }
  }

  return config;
}

export { graphFromRegex };
