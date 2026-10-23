use crate::thread::Thread;
use maud::{Markup, html};

use super::super::layouts;

pub fn thread_detail_page(thread: &Thread) -> Markup {
  let content = html! {
    div class="w-full max-w-5xl space-y-6" {
      p {
        a class="inline-flex items-center gap-2 text-sm font-medium text-muted-foreground transition-colors hover:text-foreground" href="/threads" {
          i aria-hidden="true" class="size-4" data-lucide="arrow-left" {}
          "Back to threads"
        }
      }
      header class="space-y-3" {
        h1 class="font-mono text-2xl font-semibold tracking-tight" { (thread.id()) }
        dl class="grid gap-1 text-sm text-muted-foreground sm:grid-cols-[max-content_1fr] sm:gap-x-4" {
          dt class="font-medium text-foreground" { "Workspace" }
          dd class="break-all" { (thread.workspace().display()) }
          dt class="font-medium text-foreground" { "Profile" }
          dd { (thread.profile()) }
          dt class="font-medium text-foreground" { "Messages" }
          dd { (thread.message_count()) }
        }
      }
      pre class="overflow-x-auto rounded-lg border border-border bg-card p-4 font-mono text-sm leading-6 text-card-foreground whitespace-pre-wrap" { (thread.render_transcript()) }
    }
  };
  let page = layouts::page("Thread", Some("Threads"), content);

  return page;
}
