import { basicSetup } from '@codemirror/basic-setup';
import { EditorState, Compartment } from '@codemirror/state';
import { EditorView, keymap } from '@codemirror/view';
import { qatam } from 'codemirror-lang-qatam';

import { basicLight } from 'cm6-theme-basic-light';
import { basicDark } from 'cm6-theme-basic-dark';
import { solarizedLight } from 'cm6-theme-solarized-light';
import { solarizedDark } from 'cm6-theme-solarized-dark';
import { materialDark } from 'cm6-theme-material-dark';
import { nord } from 'cm6-theme-nord';
import { gruvboxLight } from 'cm6-theme-gruvbox-light';
import { gruvboxDark } from 'cm6-theme-gruvbox-dark';

const selectTheme = document.getElementById('theme');
const divEditor = document.getElementById('editor');
const btnExecute = document.getElementById('execute');
const preOutput = document.getElementById('output');

const THEMES = {
  'basic-light': basicLight,
  'basic-dark': basicDark,
  'solarized-light': solarizedLight,
  'solarized-dark': solarizedDark,
  'material-dark': materialDark,
  nord: nord,
  'gruvbox-light': gruvboxLight,
  'gruvbox-dark': gruvboxDark,
};

const style = EditorView.theme({
  '&': {
    height: '300px',
    fontSize: '1rem',
    margin: '1rem 0',
    borderRadius: '.25rem',
  },
});

class App {
  constructor({ themes = THEMES }) {
    // themes
    const defaultTheme = localStorage.getItem('theme') || 'basic-light';
    this.themes = themes;
    this.renderSelect(defaultTheme);

    // editor
    this.view = this.configureEditor({
      doc: localStorage.getItem('code') || '',
      onChange(view) {
        localStorage.setItem('code', view.state.doc.toString());
      },
      defaultTheme,
    });

    // execute
    btnExecute.addEventListener('click', async () => {
      await this.execute(this.view.state.doc.toString());
    });
  }

  configureEditor({ doc, onChange, defaultTheme }) {
    return new EditorView({
      state: EditorState.create({
        doc,
        extensions: [
          basicSetup,
          style,
          EditorView.lineWrapping,
          qatam,
          EditorView.updateListener.of(onChange),
          (this._theme = new Compartment()).of(this.themes[defaultTheme]),
        ],
      }),
      parent: divEditor,
    });
  }

  set theme(value) {
    this.view.dispatch({
      effects: this._theme.reconfigure(this.themes[value]),
    });
    localStorage.setItem('theme', value);
  }

  renderSelect(defaultTheme) {
    selectTheme.innerHtml = Object.keys(this.themes)
      .map(theme => `<option value="${theme}">${theme}</option>`)
      .join('');
    selectTheme.value = defaultTheme;
    selectTheme.addEventListener('change', () => {
      this.theme = selectTheme.value;
    });
  }

  async execute(code) {
    if (this.isLoading) return;

    this.isLoading = true;
    preOutput.innerHTML = '...';
    const result = await fetch('/execute', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ code }),
    });

    this.isLoading = false;

    if (result.ok) {
      const output = await result.json();
      preOutput.innerHTML = `${output.stdout}\n${output.stderr}`;
    } else {
      preOutput.innerHTML = `${result.status}: ${result.statusText}`;
    }
  }
}

new App({});
