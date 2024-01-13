import {
  Decoration,
  RangeSetBuilder,
  ViewPlugin,
  hoverTooltip,
} from "@uiw/react-codemirror";
import { RegexCapture } from "regex-potata";
import palette from "./colors";

const decoration = (style: string) =>
  Decoration.mark({
    attributes: {
      style,
    },
    inclusiveStart: true,
    inclusiveEnd: false,
  });

function getMatchHighlight(captures: RegexCapture[]) {
  const decorationBuilder = new RangeSetBuilder<Decoration>();

  for (let i = 0; i < captures.length; i++) {
    const groups = captures[i].groups();

    decorationBuilder.add(
      groups[0].start,
      groups[0].end,
      decoration(`text-decoration-line: underline;
        text-underline-offset: 8px;
        text-decoration-thickness: 2px;
        text-decoration-color: ${palette[i % palette.length]};`)
    );

    for (let j = 1; j < groups.length; j++) {
      decorationBuilder.add(
        groups[j].start,
        groups[j].end,
        decoration(
          `color: #0f172a; background-color: ${
            palette[i + (j % palette.length)]
          }`
        )
      );
    }
  }

  const plugin = ViewPlugin.define(
    () => ({
      decorations: decorationBuilder.finish(),
    }),
    { decorations: (plugin) => plugin.decorations }
  );

  return plugin.extension;
}

function groupHoverTooltip(input: string, captures: RegexCapture[]) {
  return hoverTooltip((_, pos) => {
    for (let i = 0; i < captures.length; i++) {
      const groups = captures[i].groups();

      if (groups.length > 1) {
        groups.shift();
      }

      for (const group of groups) {
        const { start, end } = group;

        if (start <= pos && pos < end) {
          return {
            pos: start,
            end,
            above: true,
            create() {
              const dom = document.createElement("div");

              dom.innerHTML = `<div
              class="py-2 px-4 space-y-1
              border-[1px] border-slate-800
              shadow-sm rounded-md bg-slate-900"
            >
              <div class="font-semibold">Match n°${i + 1}</div>
              <div class="border-b-[1px] border-slate-800"></div>
              <div>
                <span
                  class="underline underline-offset-4
                  text-cyan-300 decoration-cyan-300"
                >
                  Group ${group.name()}
                </span>:
                <span class="font-semibold">${input.slice(start, end)}</span>
              </div>
              <div>
                <span class="text-slate-500">Range: </span>
                <span>${start}-${end}</span>
              </div>
            </div>`;

              return { dom };
            },
          };
        }
      }
    }

    return null;
  });
}

export { getMatchHighlight, groupHoverTooltip };
