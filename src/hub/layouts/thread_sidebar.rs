use crate::thread::Thread;
use maud::{Markup, html};

pub(crate) fn thread_sidebar(threads: &[Thread], current_thread_id: Option<&str>) -> Markup {
  let sidebar = html! {
    aside class="hidden w-60 shrink-0 flex-col border-r border-border bg-sidebar md:sticky md:top-12 md:flex md:h-[calc(100svh-3rem)] md:self-start" aria-label="Threads" {
      div class="flex h-12 shrink-0 items-center justify-between gap-2 border-b border-sidebar-border px-4" {
        span class="text-sm font-medium text-sidebar-foreground" { "Threads" }
        a aria-label="New thread" class="inline-flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground" href="/threads/new" {
          i aria-hidden="true" class="size-4" data-lucide="square-pen" {}
        }
      }
      nav class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2" aria-label="Saved threads" {
        @if threads.is_empty() {
          p class="px-2 py-3 text-sm text-muted-foreground" { "No threads yet" }
        } @else {
          @for thread in threads {
            @let is_current = current_thread_id == Some(thread.id());
            @let item_class = if is_current {
              "flex h-8 items-center gap-2 rounded-md bg-sidebar-accent px-2 text-sm font-medium text-sidebar-accent-foreground"
            } else {
              "flex h-8 items-center gap-2 rounded-md px-2 text-sm text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
            };
            a class=(item_class) href=(format!("/threads/{}", thread.id())) {
              i aria-hidden="true" class="size-4 shrink-0 text-muted-foreground" data-lucide="message-square" {}
              span class="truncate font-mono text-xs" { (thread.id()) }
            }
          }
        }
      }
    }
  };

  return sidebar;
}
