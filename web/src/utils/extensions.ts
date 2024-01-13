import {
  Decoration,
  RangeSetBuilder,
  ViewPlugin,
  hoverTooltip,
} from "@uiw/react-codemirror";
import { RegexCapture, RegexMatch } from "regex-potata";

const matchDecoration = Decoration.mark({
  class: "bg-cyan-100 text-slate-900",
  inclusiveStart: true,
  inclusiveEnd: false,
});

const groupDecoration = Decoration.mark({
  class: "underline underline-offset-8 decoration-2 decoration-cyan-300",
  inclusiveStart: true,
  inclusiveEnd: false,
});

function getMatchHighlight(matches: RegexMatch[], captures: RegexCapture[]) {
  const decorationBuilder = new RangeSetBuilder<Decoration>();

  for (const match of matches) {
    decorationBuilder.add(match.start, match.end, matchDecoration);
  }

  for (const capture of captures.slice(1)) {
    decorationBuilder.add(capture.start, capture.end, groupDecoration);
  }

  const plugin = ViewPlugin.define(
    () => ({
      decorations: decorationBuilder.finish(),
    }),
    { decorations: (plugin) => plugin.decorations }
  );

  return plugin.extension;
}

function groupHoverTooltip(captures: RegexCapture[]) {
  return hoverTooltip((_, pos) => {
    for (const capture of captures.slice(1)) {
      const { start, end } = capture;

      if (start <= pos && pos <= end) {
        return {
          pos: start,
          end,
          above: true,
          create() {
            const dom = document.createElement("div");
            dom.innerHTML = `Group <span class="font-semibold">${capture.name()}</span>`;
            dom.classList.add("py-2", "px-4", "rounded-md", "!bg-slate-600");
            return { dom };
          },
        };
      }
    }

    return null;
  });
}

export { getMatchHighlight, groupHoverTooltip };
