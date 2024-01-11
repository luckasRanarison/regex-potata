import { RegexEngine } from "regex-potata";

function dotFromRegex(regex: RegexEngine) {
  const states = regex.nfaStates();
  const endState = states.length - 1;
  const transitions = Array.from(states)
    .flatMap((state) =>
      regex
        .nfaTransition(state)
        ?.map((t) => `${state} -> ${t.end} [label="${t.toString()}"]\n`)
    )
    .join("\n");
  const dot = `
    digraph { 
      bgcolor=none; 
      graph [rankdir=LR]; 
      node [shape=circle, color=white, penwidth=2, fontcolor=white, fontname="Arial"];
      edge [color="#67e8f9", fontcolor=white, fontname="Arial"];
      ${endState} [shape=doublecircle];
      "" [shape=none]
      "" -> 0
      ${transitions}
    }`;

  return dot;
}

export { dotFromRegex };
