use crate::thread::Thread;
use maud::{Markup, html};

use super::super::layouts;

pub fn thread_detail_page(thread: &Thread, threads: &[Thread], csrf_token: &str) -> Markup {
  let content = html! {
    div class="mx-auto flex min-h-[calc(100svh-4rem-3rem)] w-full max-w-3xl flex-col" id="thread-chat" data-thread-id=(thread.id()) {
      p class="mb-6 truncate text-xs text-muted-foreground" { (thread.workspace().display()) }
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
      form class="sticky bottom-6 z-20 mt-4 rounded-xl border border-border bg-card p-2 shadow-sm relative" id="new-thread-form" {
        button aria-label="Scroll to latest message" class="absolute bottom-full left-1/2 z-30 mb-3 hidden inline-flex size-9 -translate-x-1/2 items-center justify-center rounded-md border border-border bg-card text-card-foreground shadow-sm hover:bg-accent hover:text-accent-foreground" id="thread-scroll-bottom" type="button" {
          i aria-hidden="true" class="size-4" data-lucide="arrow-down" {}
        }
        label class="sr-only" for="thread-message" { "Message" }
        div class="flex items-center gap-2" {
          input autofocus autocomplete="off" class="flex h-9 min-w-0 flex-1 bg-transparent px-2 text-sm outline-none placeholder:text-muted-foreground" id="thread-message" name="message" placeholder="Ask Kermet anything..." type="text" {}
          button aria-label="Send message" class="inline-flex size-8 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground" id="thread-submit" type="submit" {
            i aria-hidden="true" class="size-4" data-lucide="arrow-up" {}
          }
        }
      }
    }
  };
  let page = layouts::thread_page("Thread", content, threads, Some(thread.id()));

  return page;
}
