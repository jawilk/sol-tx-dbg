<template>
  <codemirror
    class="editor"
    ref="cm"
    v-model="code"
    :style="{ height: '100%', width: '100%' }"
    :extensions="extensions"
    @ready="handleReady"
    @change="log('change', $event)"
    @focus="log('focus', $event)"
  />
</template>

<script>
import { ref, shallowRef } from "vue";
import { Codemirror } from "vue-codemirror";
import { rust } from "@codemirror/lang-rust";
import { oneDark } from "../one-dark.js";

import {
  EditorState,
  StateField,
  StateEffect,
  RangeSet,
} from "@codemirror/state";
import {
  EditorView,
  Decoration,
  lineNumbers,
  gutter,
  GutterMarker,
} from "@codemirror/view";

const addLineHighlight = StateEffect.define();

const lineHighlightField = StateField.define({
  create() {
    return Decoration.none;
  },
  update(lines, tr) {
    lines = lines.map(tr.changes);
    for (let e of tr.effects) {
      if (e.is(addLineHighlight)) {
        lines = Decoration.none;
        lines = lines.update({ add: [lineHighlightMark.range(e.value)] });
      }
    }
    return lines;
  },
  provide: (f) => EditorView.decorations.from(f),
});

const lineHighlightMark = Decoration.line({
  attributes: { style: "background-color: #595910" },
});

const breakpointEffect = StateEffect.define({
  map: (val, mapping) => ({ pos: mapping.mapPos(val.pos), on: val.on }),
});

const breakpointState = StateField.define({
  create() {
    return RangeSet.empty;
  },
  update(set, transaction) {
    set = set.map(transaction.changes);

    for (let e of transaction.effects) {
      if (e.is(breakpointEffect)) {
        if (e.value.on)
          set = set.update({ add: [breakpointMarker.range(e.value.pos)] });
        else set = set.update({ filter: (from) => from != e.value.pos });
      }
    }

    return set;
  },
});

const breakpointMarker = new (class extends GutterMarker {
  toDOM() {
    return document.createTextNode("●");
  }
})();

export default {
  name: "EditorComponent",
  components: {
    Codemirror,
  },
  props: {
    files_url: String,
    program_id: String,
    editorState: Object,
  },
  data() {
    return {
      curFile: "",
      prevBreakpointsEditor: [],
      // Hack to not scroll to top of doc on first gutter click
      is_first_breakpoint: true,
    };
  },
  methods: {
    highlightLine(line) {
      const docPosition = this.view.state.doc.line(line).from;
      this.view.dispatch(
        this.getCodemirrorStates().state.update({
          selection: { anchor: docPosition, head: docPosition },
          effects: [
            addLineHighlight.of(docPosition),
            EditorView.scrollIntoView(docPosition, {
              y: "center",
            }),
          ],
        })
      );
    },
    async parseFile(url) {
      try {
        const fetchResponse = await fetch(url);
        return fetchResponse;
      } catch (e) {
        console.log(e);
      }
    },
    toggleBreakpoint(pos) {
      let breakpoints = this.view.state.field(breakpointState);
      let hasBreakpoint = false;
      breakpoints.between(pos, pos, () => {
        hasBreakpoint = true;
      });
      let effects;
      if (this.is_first_breakpoint) {
        this.is_first_breakpoint = false;
        effects = [
          breakpointEffect.of({ pos, on: !hasBreakpoint }),
          EditorView.scrollIntoView(pos, {
            y: "center",
          }),
        ];
      } else effects = [breakpointEffect.of({ pos, on: !hasBreakpoint })];

      this.view.dispatch(
        this.getCodemirrorStates().state.update({
          effects: effects,
        })
      );
    },
    handleBreakpoints() {
      if (this.editorState.breakpoints === undefined) return;
      if (this.prevBreakpointsEditor === null) {
        this.editorState.breakpoints.forEach((l) => {
          const docPosition = this.view.state.doc.line(l).from;
          this.toggleBreakpoint(docPosition);
        });

        this.prevBreakpointsEditor = this.editorState.breakpoints;
        return;
      }
      const difference = [
        // Add
        ...this.editorState.breakpoints.filter(
          (item) => !this.prevBreakpointsEditor.includes(item)
        ),
        // Delete
        ...this.prevBreakpointsEditor.filter(
          (item) => !this.editorState.breakpoints.includes(item)
        ),
      ];
      if (difference.length === 0) return;
      difference.forEach((l) => {
        const docPosition = this.view.state.doc.line(l).from;
        this.toggleBreakpoint(docPosition);
      });

      this.prevBreakpointsEditor = JSON.parse(
        JSON.stringify(this.editorState.breakpoints)
      );
    },
  },
  watch: {
    editorState() {
      if (this.editorState.file !== this.curFile) {
        this.curFile = this.editorState.file;
        this.prevBreakpointsEditor = [];
        this.parseFile(
          this.files_url +
            "code/" +
            this.program_id +
            "/" +
            this.editorState.file
        )
          .then((response) => response.text())
          .then((txt) => {
            let newState = EditorState.create({
              doc: txt,
              readOnly: true,
              extensions: [
                EditorView.contentAttributes.of({ contenteditable: false }),
                this.extensions,
                lineNumbers(),
              ],
            });
            this.view.setState(newState);
            if (this.editorState.breakpoints !== undefined)
              this.handleBreakpoints();

            if (this.editorState.line !== undefined)
              this.highlightLine(this.editorState.line);

            this.is_first_breakpoint = true;
          });
      } else {
        if (this.editorState.breakpoints !== undefined)
          this.handleBreakpoints();
        if (
          this.editorState.updateType !== "breakpoints" &&
          this.editorState.line !== undefined
        )
          this.highlightLine(this.editorState.line);
      }
    },
  },
  setup(props, context) {
    const breakpointGutter = [
      breakpointState,
      gutter({
        class: "cm-breakpoint-gutter",
        markers: (v) => v.state.field(breakpointState),
        initialSpacer: () => breakpointMarker,
        domEventHandlers: {
          mousedown(view, line) {
            context.emit(
              "toggleBreakpoints",
              "",
              view.state.doc.lineAt(line.from).number
            );
            return true;
          },
        },
      }),
      EditorView.baseTheme({
        ".cm-breakpoint-gutter .cm-gutterElement": {
          color: "red",
          paddingLeft: "5px",
        },
        ".cm-breakpoint-gutter": {
          cursor: "pointer",
        },
      }),
    ];
    const view = shallowRef();
    const handleReady = (payload) => {
      view.value = payload.view;
    };

    const code = ref(" ");
    const extensions = [
      EditorView.contentAttributes.of({ contenteditable: false }),
      lineHighlightField,
      rust(),
      oneDark,
      ...breakpointGutter,
    ];

    const getCodemirrorStates = () => {
      const state = view.value.state;
      const doc = state.doc;
      return {
        state,
        doc,
      };
    };

    return {
      view,
      getCodemirrorStates,
      code,
      extensions,
      handleReady,
      log: console.log,
    };
  },
};
</script>

<style scoped>
.editor {
  font-size: 12px;
}
</style>