use maud::{Markup, html};

use super::super::layouts;

pub fn home_page() -> Markup {
  let content = html! {
    div class="w-full max-w-5xl" {
      h1 class="text-2xl font-semibold tracking-tight" { "Hello" }
    }
  };
  let page = layouts::page("Kermet Hub", None, content);

  return page;
}
