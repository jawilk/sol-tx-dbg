<template>
  <div class="editor-container">
  <codemirror
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
    @blur="log('blur', $event)"
  />
</div>
</template>

<script>
  import { ref, shallowRef } from 'vue'
  import { Codemirror } from 'vue-codemirror'
  import { rust } from '@codemirror/lang-rust'
  import { oneDark } from '@codemirror/theme-one-dark'
import { EditorState, StateField, StateEffect} from '@codemirror/state';
import {EditorView, Decoration, lineNumbers} from '@codemirror/view';


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
        lines = lines.update({add: [lineHighlightMark.range(e.value)]});
      }
    }
    return lines;
  },
  provide: (f) => EditorView.decorations.from(f),
});

const lineHighlightMark = Decoration.line({
  attributes: {style: 'background-color: #595910'},
});

  export default {
    name: 'EditorComponent',
    components: {
      Codemirror
    },
    props: {
      'fileName': String,
      'next': Boolean,
    },
    methods: {
 highlightLine(line) {
    console.log("highlightLine", line)
  
    const docPosition2 = this.view.state.doc.line(line).from;
    this.view.dispatch({effects: addLineHighlight.of(docPosition2)});
  },
  async parseFile(url) {
   try {
     const fetchResponse = await fetch(
       url
     );
      return fetchResponse;
   } catch (ex) {
     console.log("Error in fetch");
   }
}
    },
    watch: {
      next() {
        console.log("NEXT", this.line)
        this.highlightLine(this.line++)
      },
        fileName() {
            console.log('CHANGED', this.fileName);
            this.parseFile('http://localhost:8002/'+this.fileName).then(response => response.text())
              .then(txt => {
              let newState = EditorState.create({
  doc: txt,
  extensions: [this.extensions, lineNumbers()],
  // selection: EditorSelection.cursor(exercise2.cursorStart),
});
             this.view.setState(newState);
              // this.view.dispatch({changes: {from: 0, to: this.view.state.doc.length, insert: txt}})
                this.highlightLine(6);          
            })
          }
    },
    setup() {
       // Codemirror EditorView instance ref
      const view = shallowRef()
      const handleReady = (payload) => {
        view.value = payload.view
      }

      const code = ref(`console.log('Hello, world!')`)
      const extensions = [lineHighlightField, rust(), oneDark]

     

      // Status is available at all times via Codemirror EditorView
      const getCodemirrorStates = () => {
        const state = view.value.state
        // const ranges = state.selection.ranges
        // const selected = ranges.reduce((r, range) => r + range.to - range.from, 0)
        // const cursor = ranges[0].anchor
        // const length = state.doc.length
        const doc = state.doc
        // more state info ...
        // return ...
        return {
          state,
          doc
        }
      }

      return {
        view,
        getCodemirrorStates,
        code,
        extensions,
        handleReady,
        log: console.log,
        line: 0,
      }
    },
  }
</script>

<style scoped>

.editor-container {
  height: 100%;
  width: 100%;
  overflow: scroll;
}

</style>