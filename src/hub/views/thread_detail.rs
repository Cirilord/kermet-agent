use crate::thread::Thread;
use maud::{Markup, html};

use super::super::layouts;

pub fn thread_detail_page(thread: &Thread, csrf_token: &str) -> Markup {
  let content = html! {
    div class="mx-auto flex min-h-[calc(100svh-4rem-3rem)] w-full max-w-3xl flex-col py-8 sm:py-12" id="thread-chat" data-thread-id=(thread.id()) {
      div class="mb-6 flex items-center justify-between gap-4" {
        a class="inline-flex items-center gap-2 text-sm font-medium text-muted-foreground transition-colors hover:text-foreground" href="/threads" {
          i aria-hidden="true" class="size-4" data-lucide="arrow-left" {}
          "Back to threads"
        }
        span class="truncate text-xs text-muted-foreground" { (thread.workspace().display()) }
      }
      div class="flex flex-1 flex-col gap-3" id="thread-transcript" aria-live="polite" {
        @for message in thread.messages() {
          @if message.is_user() {
            article class="ml-auto max-w-[85%] rounded-lg bg-primary px-3 py-2 text-sm text-primary-foreground" {
              p class="whitespace-pre-wrap" { (message.content()) }
            }
          } @else {
            article class="mr-auto max-w-[85%] rounded-lg border border-border bg-card px-3 py-2 text-sm text-card-foreground" {
              p class="whitespace-pre-wrap" { (message.content()) }
            }
          }
        }
      }
      input id="workspace-csrf-token" type="hidden" value=(csrf_token) {}
      form class="rounded-xl border border-border bg-card p-3 shadow-sm" id="new-thread-form" {
        label class="sr-only" for="thread-message" { "Message" }
        textarea autofocus class="flex min-h-24 w-full resize-none bg-transparent px-1 py-2 text-sm outline-none placeholder:text-muted-foreground" id="thread-message" name="message" placeholder="Ask Kermet anything..." {}
        div class="flex items-center justify-between border-t border-border pt-3" {
          span class="text-xs text-muted-foreground" id="thread-status" { "Ready for your next message." }
          button aria-label="Send message" class="inline-flex size-8 items-center justify-center rounded-md bg-primary text-primary-foreground" id="thread-submit" type="submit" {
            i aria-hidden="true" class="size-4" data-lucide="arrow-up" {}
          }
        }
      }
    }
  };
  let page = layouts::page("Thread", Some("Threads"), content);

  return page;
}
