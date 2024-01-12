import { Decoration, RangeSetBuilder, ViewPlugin } from "@uiw/react-codemirror";
import { RegexMatch } from "regex-potata";

const matchDecoration = Decoration.mark({
  class: "bg-cyan-100 text-slate-900",
  inclusiveStart: true,
  inclusiveEnd: false,
});

function getMatchHighlight(matches: RegexMatch[]) {
  const decorationBuilder = new RangeSetBuilder<Decoration>();

  for (const match of matches) {
    decorationBuilder.add(match.start, match.end, matchDecoration);
  }

  const plugin = ViewPlugin.define(
    () => ({
      decorations: decorationBuilder.finish(),
    }),
    { decorations: (plugin) => plugin.decorations }
  );

  return plugin.extension;
}

export { getMatchHighlight };
