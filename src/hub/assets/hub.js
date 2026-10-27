$(function () {
  lucide.createIcons();

  const $shell = $('#hub-shell');
  const $sidebar = $('#hub-sidebar');
  const $backdrop = $('#sidebar-backdrop');
  const $workspaceTrigger = $('#workspace-trigger');
  const $workspaceMenu = $('#workspace-menu');
  const $workspaceTriggerLabel = $('#workspace-trigger-label');
  const $workspaceDialog = $('#workspace-dialog');
  const $workspaceDialogBackdrop = $('#workspace-dialog-backdrop');
  const $workspacePath = $('#workspace-path');
  const $workspaceCsrfToken = $('#workspace-csrf-token');
  const $workspacePickerTrigger = $('#workspace-picker-trigger');
  const $workspacePickerStatus = $('#workspace-picker-status');
  const $workspaceId = $('#workspace-id');
  const $threadForm = $('#new-thread-form');
  const $threadMessage = $('#thread-message');
  const $threadSubmit = $('#thread-submit');
  const $threadTranscript = $('#thread-transcript');
  const $threadScrollBottom = $('#thread-scroll-bottom');
  let threadId = null;
  let eventSource = null;
  let assistantMessage = null;

  function closeSidebar() {
    $sidebar.addClass('hidden').removeClass('flex');
    $backdrop.addClass('hidden');
  }

  function closeWorkspaceMenu() {
    $workspaceMenu.addClass('hidden');
    $workspaceTrigger.attr('aria-expanded', 'false');
  }

  function openWorkspaceDialog() {
    closeWorkspaceMenu();
    $workspaceDialog.removeClass('hidden');
    $workspaceDialogBackdrop.removeClass('hidden').attr('aria-hidden', 'false');
    window.setTimeout(function () {
      $workspacePath.trigger('focus');
    });
  }

  function closeWorkspaceDialog() {
    $workspaceDialog.addClass('hidden');
    $workspaceDialogBackdrop.addClass('hidden').attr('aria-hidden', 'true');
    $workspaceTrigger.trigger('focus');
  }

  function updateThreadScrollButton() {
    if (threadId === null || !$threadTranscript.length) {
      $threadScrollBottom.addClass('hidden');
      return;
    }

    const distanceToBottom = document.documentElement.scrollHeight - window.innerHeight - window.scrollY;

    $threadScrollBottom.toggleClass('hidden', distanceToBottom <= 24);
  }

  function scrollThreadToBottom(behavior) {
    if (!$threadTranscript.length) {
      return;
    }

    window.requestAnimationFrame(function () {
      window.scrollTo({ top: document.documentElement.scrollHeight, behavior: behavior });
      updateThreadScrollButton();
    });
  }

  async function chooseWorkspaceFolder() {
    const csrfToken = $workspaceCsrfToken.val();
    $workspacePickerTrigger.prop('disabled', true);
    $workspacePickerStatus.text('Opening the native folder picker…');

    try {
      const response = await window.fetch('/api/workspaces/picker', {
        method: 'POST',
        credentials: 'same-origin',
        headers: {
          'X-Kermet-CSRF': csrfToken,
        },
      });

      if (response.status === 204) {
        $workspacePickerStatus.text('No folder was selected.');
        return;
      }

      if (!response.ok) {
        $workspacePickerStatus.text('Kermet could not open the folder picker.');
        return;
      }

      const workspace = await response.json();
      $workspacePath.val(workspace.path);
      $workspaceId.val(workspace.id);
      $workspaceTriggerLabel.text(workspace.path);
      $workspacePickerStatus.text('Workspace registered.');
      closeWorkspaceDialog();
    } catch (_) {
      $workspacePickerStatus.text('Kermet could not open the folder picker.');
      return;
    } finally {
      $workspacePickerTrigger.prop('disabled', false);
    }
  }

  function appendMessage(role, message) {
    const isUser = role === 'user';
    const $message = $('<article>')
      .addClass(isUser ? 'ml-auto max-w-[85%] rounded-lg bg-primary px-3 py-2 text-sm text-primary-foreground' : 'mr-auto max-w-[85%] rounded-lg border border-border bg-card px-3 py-2 text-sm text-card-foreground');
    const $content = $('<p>').addClass('whitespace-pre-wrap').text(message);

    $message.append($content);
    $threadTranscript.removeClass('hidden').append($message);
    scrollThreadToBottom();

    return $content;
  }

  function appendTurnEvent(message) {
    const $event = $('<p>')
      .addClass('self-center text-xs text-muted-foreground')
      .attr('role', 'status')
      .text(message);

    $threadTranscript.removeClass('hidden').append($event);
    scrollThreadToBottom();
  }

  function setTurnPending() {
    $threadSubmit.prop('disabled', true);
    $threadMessage.prop('disabled', true);
  }

  function finishTurn() {
    assistantMessage = null;
    $threadSubmit.prop('disabled', false);
    $threadMessage.prop('disabled', false).trigger('focus');
  }

  async function startInitialTurn() {
    try {
      const response = await window.fetch(`/api/threads/${threadId}/initial-turn`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'X-Kermet-CSRF': $workspaceCsrfToken.val() },
      });

      if (!response.ok) {
        appendTurnEvent('Kermet could not start the thread.');
        finishTurn();
      }
    } catch (_) {
      appendTurnEvent('Kermet could not start the thread.');
      finishTurn();
    }
  }

  function connectThreadEvents(id, onConnected) {
    eventSource = new EventSource(`/api/threads/${id}/events`);
    eventSource.addEventListener('connected', function () {
      if (onConnected !== null) {
        onConnected();
      }
    }, { once: true });
    eventSource.addEventListener('thinking', function () {
      appendTurnEvent('Thinking…');
    });
    eventSource.addEventListener('tool_call', function (event) {
      appendTurnEvent(`Calling ${event.data}`);
    });
    eventSource.addEventListener('tool_execution', function (event) {
      appendTurnEvent(`Running ${event.data}`);
    });
    eventSource.addEventListener('text_delta', function (event) {
      if (assistantMessage === null) {
        assistantMessage = appendMessage('assistant', '');
      }

      assistantMessage.text(`${assistantMessage.text()}${event.data}`);
      scrollThreadToBottom();
    });
    eventSource.addEventListener('completed', function () {
      finishTurn();
    });
    eventSource.addEventListener('failed', function () {
      appendTurnEvent('Kermet could not complete this turn.');
      finishTurn();
    });
  }

  function connectExistingThread() {
    const existingThreadId = $('#thread-chat').attr('data-thread-id');

    if (!existingThreadId) {
      return;
    }

    threadId = existingThreadId;
    connectThreadEvents(threadId, null);
    scrollThreadToBottom();
  }

  async function sendMessage(message) {
    setTurnPending();
    appendMessage('user', message);
    appendTurnEvent('Sending message…');
    $threadMessage.val('');

    try {
      const response = await window.fetch(`/api/threads/${threadId}/messages`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json', 'X-Kermet-CSRF': $workspaceCsrfToken.val() },
        body: JSON.stringify({ message: message }),
      });

      if (!response.ok) {
        appendTurnEvent('Kermet could not send the message.');
        finishTurn();
      }
    } catch (_) {
      appendTurnEvent('Kermet could not send the message.');
      finishTurn();
    }
  }

  $('#sidebar-toggle').on('click', function () {
    if (window.matchMedia('(min-width: 768px)').matches) {
      $shell.toggleClass('sidebar-collapsed');
      return;
    }

    $sidebar.removeClass('hidden').addClass('flex');
    $backdrop.removeClass('hidden');
  });

  $('#sidebar-close, #sidebar-backdrop').on('click', closeSidebar);

  $workspaceTrigger.on('click', function () {
    const isExpanded = $workspaceTrigger.attr('aria-expanded') === 'true';
    const nextState = !isExpanded;

    $workspaceMenu.toggleClass('hidden', !nextState);
    $workspaceTrigger.attr('aria-expanded', nextState);
  });

  $workspaceMenu.on('click', '[data-workspace-path]', function () {
    $workspaceTriggerLabel.text($(this).attr('data-workspace-path'));
    $workspaceId.val($(this).attr('data-workspace-id'));
    closeWorkspaceMenu();
  });
  $('#new-workspace-trigger').on('click', openWorkspaceDialog);
  $('#workspace-dialog-cancel, #workspace-dialog-backdrop').on('click', closeWorkspaceDialog);
  $workspacePickerTrigger.on('click', chooseWorkspaceFolder);
  connectExistingThread();
  $threadScrollBottom.on('click', function () {
    scrollThreadToBottom('smooth');
  });
  $(window).on('scroll resize', updateThreadScrollButton);
  $threadForm.on('submit', async function (event) {
    event.preventDefault();
    const workspaceId = $workspaceId.val();
    const message = $threadMessage.val().trim();

    if (!message) {
      return;
    }

    if (threadId !== null) {
      await sendMessage(message);
      return;
    }

    if (!workspaceId) {
      return;
    }

    setTurnPending();

    try {
      const response = await window.fetch('/api/threads', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json', 'X-Kermet-CSRF': $workspaceCsrfToken.val() },
        body: JSON.stringify({ workspace_id: workspaceId, message: message }),
      });
      if (!response.ok) {
        appendTurnEvent('Kermet could not create the thread.');
        finishTurn();
        return;
      }
      const thread = await response.json();
      threadId = thread.id;
      window.history.replaceState(null, '', `/threads/${thread.id}`);
      appendMessage('user', message);
      appendTurnEvent('Starting thread…');
      $threadMessage.val('');
      connectThreadEvents(thread.id, function () {
        startInitialTurn();
      });
    } catch (_) {
      appendTurnEvent('Kermet could not create the thread.');
      finishTurn();
    }
  });

  $(document).on('click', function (event) {
    if (!$(event.target).closest('#workspace-selector').length) {
      closeWorkspaceMenu();
    }
  });
  $(document).on('keydown', function (event) {
    if (event.key === 'Escape' && !$workspaceDialog.hasClass('hidden')) {
      closeWorkspaceDialog();
    }
  });
});
