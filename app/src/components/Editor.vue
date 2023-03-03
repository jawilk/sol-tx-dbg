<template>
  <codemirror
    class="editor"
    ref="cm"
    v-model="code"
    placeholder="Code goes here..."
    :style="{ height: '100%', width: '100%' }"
    :autofocus="true"
    :indent-with-tab="true"
    :tab-size="2"
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
// import { oneDark } from "@codemirror/theme-one-dark";
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
        if (e.value.on) {
          set = set.update({ add: [breakpointMarker.range(e.value.pos)] });
        } else set = set.update({ filter: (from) => from != e.value.pos });
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
      is_first: true,
    };
  },
  methods: {
    highlightLine(line) {
      console.log("highlightLine", line);
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
      } catch (ex) {
        console.log("Error in fetch");
      }
    },
    toggleBreakpoint(pos) {
      // is_first hack for focus/scroll
      console.log("is_first !!!!", this.is_first);
      let breakpoints = this.view.state.field(breakpointState);
      let hasBreakpoint = false;
      breakpoints.between(pos, pos, () => {
        hasBreakpoint = true;
      });
      const effects = [breakpointEffect.of({ pos, on: !hasBreakpoint })];
      if (this.is_first) {
        effects.push(
          EditorView.scrollIntoView(pos, {
            y: "center",
          })
        );
      }
      this.view.dispatch(
        this.getCodemirrorStates().state.update({
          effects: effects,
        })
        // {scrollIntoView: 'always'},
      );
      this.is_first = false;
    },
    handleBreakpoints() {
      console.log("breakpointsEditor!", this.editorState.breakpoints, this.prevBreakpointsEditor);
      if (this.editorState.breakpoints === undefined) {
        return;
      }
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
      difference.forEach((l) => {
        const docPosition = this.view.state.doc.line(l).from;
        this.toggleBreakpoint(docPosition);
      });
      console.log("difference", difference);

      this.prevBreakpointsEditor = JSON.parse(
        JSON.stringify(this.editorState.breakpoints)
      );
    },
  },
  watch: {
    editorState() {
      console.log("(editor) file WATCHER", this.editorState, this.curFile);
      this.is_first = true;
      if (this.editorState.file !== this.curFile) {
        this.prevBreakpointsEditor = [];
        console.log("(editor) NEW FILE", this.editorState);
        this.parseFile(
          this.files_url +
            "/code/" +
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
            if (this.editorState.breakpoints !== undefined) {
              this.handleBreakpoints();
            }
            if (this.editorState.line !== undefined) {
              this.highlightLine(this.editorState.line);
            }
          });
      } else {
        console.log("(editor) ELSE No NEW FILE", this.editorState);

        if (this.editorState.breakpoints !== undefined) {
          this.handleBreakpoints();
        }
        if (this.editorState.line !== undefined) {
          this.highlightLine(this.editorState.line);
        }
      }
      this.curFile = this.editorState.file;
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
            //return true;
          },
        },
      }),
      EditorView.baseTheme({
        ".cm-breakpoint-gutter .cm-gutterElement": {
          color: "red",
          cursor: "pointer",
          paddingLeft: "5px",
        },
        // ".cm-gutterElement:hover": {
        //   color: "red",
        // },
      }),
    ];
    // Codemirror EditorView instance ref
    const view = shallowRef();
    const handleReady = (payload) => {
      console.log("pay", payload);
      view.value = payload.view;
      // payload.on("gutterClick", handleGutterClick);
    };

    // function handleGutterClick(instance, line, gutter, clickEvent) {
    //   // Your gutter click event handling logic goes here
    //   console.log("breakpointevent2", line, gutter, clickEvent);
    // }

    const code = ref(" ");
    const extensions = [
      EditorView.contentAttributes.of({ contenteditable: false }),
      lineHighlightField,
      rust(),
      oneDark,
      ...breakpointGutter,
    ];

    // Status is available at all times via Codemirror EditorView
    const getCodemirrorStates = () => {
      const state = view.value.state;
      // const ranges = state.selection.ranges
      // const selected = ranges.reduce((r, range) => r + range.to - range.from, 0)
      // const cursor = ranges[0].anchor
      // const length = state.doc.length
      const doc = state.doc;
      // more state info ...
      // return ...
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