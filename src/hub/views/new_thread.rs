use maud::{Markup, html};

use super::super::layouts;

pub fn new_thread_page() -> Markup {
  let content = html! {
    div class="flex min-h-[calc(100svh-4rem-3rem)] w-full flex-col" {
      div class="mx-auto flex w-full max-w-3xl flex-1 flex-col py-8 sm:py-12" {
        div class="flex flex-1 flex-col justify-center pb-12" {
          div class="space-y-2 text-center" {
            div class="mx-auto flex size-10 items-center justify-center rounded-lg border border-border bg-card text-card-foreground" {
              i aria-hidden="true" class="size-5" data-lucide="message-square-plus" {}
            }
            h1 class="text-2xl font-semibold tracking-tight" { "Start a new thread" }
            p class="text-sm text-muted-foreground" { "Ask Kermet to work in your workspace." }
          }
        }
        form class="rounded-xl border border-border bg-card p-3 shadow-sm" {
          label class="sr-only" for="thread-message" { "Message" }
          textarea autofocus class="flex min-h-24 w-full resize-none bg-transparent px-1 py-2 text-sm outline-none placeholder:text-muted-foreground" id="thread-message" name="message" placeholder="Ask Kermet anything..." {}
          div class="flex items-center justify-between border-t border-border pt-3" {
            span class="text-xs text-muted-foreground" { "Thread creation will be available here." }
            button aria-label="Send message" class="inline-flex size-8 items-center justify-center rounded-md bg-primary text-primary-foreground opacity-50" disabled type="button" {
              i aria-hidden="true" class="size-4" data-lucide="arrow-up" {}
            }
          }
        }
      }
    }
  };
  let page = layouts::page("New thread", Some("Threads"), content);

  return page;
}
