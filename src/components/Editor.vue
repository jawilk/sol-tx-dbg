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

function toggleBreakpoint(view, pos) {
  let breakpoints = view.state.field(breakpointState);
  let hasBreakpoint = false;
  breakpoints.between(pos, pos, () => {
    hasBreakpoint = true;
  });
  view.dispatch({
    effects: breakpointEffect.of({ pos, on: !hasBreakpoint }),
  });
}

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
    program_id: String,
    file: Object,
    breakpointsEditor: Object,
    breakpointsEditorRemove: null,
  },
  data() {
    return {
      curFile: {},
    }
  },
  mounted() {

    this.$nextTick(function () {
    const element = document.querySelector('.cm-breakpoint-gutter')
    console.log("HERE", element)
    element.addEventListener('click', () => {
      this.breakpointEvent();
    });
      });
  },
  methods: {
    gutterClick(cm, n) {
      console.log("gutterClick", cm, n);
    },
    handleGutterClick(instance, line, gutter, clickEvent) {
    // Your gutter click event handling logic goes here
    console.log("breakpointevent2", instance, line, gutter, clickEvent);
    },
    breakpointEvent() {
      console.log("breakpointevent");
    },
    highlightLine(line) {
      console.log("highlightLine", line);
      const docPosition = this.view.state.doc.line(line).from;
      this.view.dispatch(this.getCodemirrorStates().state.update({ selection: { anchor: docPosition, head: docPosition }, 
      effects: [addLineHighlight.of(docPosition), EditorView.scrollIntoView(docPosition, {
     y: 'center', })] }));
    },
    addBreakpoints() {
        this.breakpointsEditor.forEach((l) => {
          const docPosition = this.view.state.doc.line(l).from;
          this.view.dispatch({
            effects: breakpointEffect.of({ pos: docPosition, on: true }),
          });
        });
    },
    async parseFile(url) {
      try {
        const fetchResponse = await fetch(url);
        return fetchResponse;
      } catch (ex) {
        console.log("Error in fetch");
      }
    },
  },
  watch: {
    // Event from breakpoint panel
    breakpointsEditorRemove() {
      console.log("BREAKPOINTS", this.breakpointsEditorRemove);
        const docPosition = this.view.state.doc.line(this.breakpointsEditorRemove).from;
        this.view.dispatch({
          effects: breakpointEffect.of({ pos: docPosition, on: false }),
      });
    },
    file() {
      if (this.file.name !== this.curFile.name) {
      console.log("FILE", this.file);
      this.parseFile("http://localhost:8003/code/" + this.program_id + '/' + this.file.name)
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
          this.view.setState(newState)
          if (this.file.line !== undefined) {
            console.log("HEREE", this.file.line)
            this.highlightLine(this.file.line);
      }
      });
        this.curFile = this.file;
    } else {
        if (this.file.line !== undefined) {
          this.highlightLine(this.file.line);
      }
    }
      if (this.breakpointsEditor !== undefined) {
          this.addBreakpoints();
      }
  }
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
        console.log("LINE:",line);
        context.emit('breakpoint', view.state.doc.lineAt(line.from).number);
        toggleBreakpoint(view, line.from);
        return true;
      },
    },
  }),
  EditorView.baseTheme({
    ".cm-breakpoint-gutter .cm-gutterElement": {
      color: "red",
      cursor: "pointer",
      paddingLeft: "5px",
    },
    ".cm-gutterElement:hover": {
      color: "red",
    },
  }),
];
    // Codemirror EditorView instance ref
    const view = shallowRef();
    const handleReady = (payload) => {
      console.log("pay",payload)
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