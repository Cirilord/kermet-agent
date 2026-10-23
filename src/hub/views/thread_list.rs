use crate::thread::Thread;
use maud::{Markup, html};

use super::super::layouts;

pub fn thread_list_page(threads: &[Thread]) -> Markup {
  let content = html! {
    div class="w-full max-w-5xl space-y-6" {
      header class="space-y-1" {
        h1 class="text-2xl font-semibold tracking-tight" { "Threads" }
        p class="text-sm text-muted-foreground" { "Persisted conversations" }
      }
      @if threads.is_empty() {
        p class="rounded-lg border border-dashed border-border p-6 text-sm text-muted-foreground" { "No threads have been created yet." }
      } @else {
        section class="space-y-2" {
          @for thread in threads {
            a class="block rounded-lg border border-border bg-card p-4 transition-colors hover:bg-accent" href=(format!("/threads/{}", thread.id())) {
              strong class="font-mono text-sm font-semibold text-card-foreground" { (thread.id()) }
              dl class="mt-3 grid gap-1 text-sm text-muted-foreground sm:grid-cols-[max-content_1fr] sm:gap-x-4" {
                dt class="font-medium text-card-foreground" { "Workspace" }
                dd class="break-all" { (thread.workspace().display()) }
                dt class="font-medium text-card-foreground" { "Profile" }
                dd { (thread.profile()) }
                dt class="font-medium text-card-foreground" { "Messages" }
                dd { (thread.message_count()) }
              }
            }
          }
        }
      }
    }
  };
  let page = layouts::page("Threads", Some("Threads"), content);

  return page;
}
