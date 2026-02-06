import VPLTheme from '@lando/vitepress-theme-default-plus'
import MermaidDiagram from './MermaidDiagram.vue'
import './custom.css'

export default {
  ...VPLTheme,
  enhanceApp(ctx: any) {
    VPLTheme.enhanceApp(ctx)
    ctx.app.component('Mermaid', MermaidDiagram)
  },
}
