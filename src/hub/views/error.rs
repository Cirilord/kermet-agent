use maud::{Markup, html};

use super::super::layouts;

pub fn error_page(title: &str, message: &str) -> Markup {
  let content = html! {
    div class="w-full max-w-5xl space-y-6" {
      div class="space-y-2" {
        h1 class="text-2xl font-semibold tracking-tight" { (title) }
        p class="text-sm text-muted-foreground" { (message) }
      }
      p {
        a class="inline-flex rounded-lg border border-border px-3 py-2 text-sm font-medium transition-colors hover:bg-accent" href="/threads" { "Back to threads" }
      }
    }
  };
  let page = layouts::page(title, Some("Error"), content);

  return page;
}
