$(function () {
  lucide.createIcons();

  const $shell = $('#hub-shell');
  const $sidebar = $('#hub-sidebar');
  const $backdrop = $('#sidebar-backdrop');
  const $threadsToggle = $('#threads-toggle');
  const $threadsMenu = $('#threads-menu');
  const $threadsChevron = $('#threads-chevron');
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
  const $threadStatus = $('#thread-status');
  const $threadSubmit = $('#thread-submit');
  const $threadTranscript = $('#thread-transcript');
  let threadId = null;
  let eventSource = null;
  let assistantMessage = null;

  function closeSidebar() {
    $sidebar.addClass('hidden').removeClass('flex');
    $backdrop.addClass('hidden');
  }

  function toggleThreadsMenu() {
    const isExpanded = $threadsToggle.attr('aria-expanded') === 'true';
    const nextState = !isExpanded;

    $threadsToggle.attr('aria-expanded', nextState);
    $threadsMenu.toggleClass('hidden', !nextState);
    $threadsChevron.toggleClass('rotate-90', nextState);
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

    return $content;
  }

  function setTurnPending(status) {
    $threadSubmit.prop('disabled', true);
    $threadMessage.prop('disabled', true);
    $threadStatus.text(status);
  }

  function finishTurn(status) {
    assistantMessage = null;
    $threadSubmit.prop('disabled', false);
    $threadMessage.prop('disabled', false).trigger('focus');
    $threadStatus.text(status);
  }

  async function startInitialTurn() {
    try {
      const response = await window.fetch(`/api/threads/${threadId}/initial-turn`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'X-Kermet-CSRF': $workspaceCsrfToken.val() },
      });

      if (!response.ok) {
        finishTurn('Kermet could not start the thread.');
      }
    } catch (_) {
      finishTurn('Kermet could not start the thread.');
    }
  }

  function connectThreadEvents(id, onConnected) {
    eventSource = new EventSource(`/api/threads/${id}/events`);
    eventSource.addEventListener('connected', function () {
      onConnected();
    }, { once: true });
    eventSource.addEventListener('thinking', function () {
      $threadStatus.text('Thinking…');
    });
    eventSource.addEventListener('tool_call', function (event) {
      $threadStatus.text(`Calling ${event.data}`);
    });
    eventSource.addEventListener('tool_execution', function (event) {
      $threadStatus.text(`Running ${event.data}`);
    });
    eventSource.addEventListener('text_delta', function (event) {
      if (assistantMessage === null) {
        assistantMessage = appendMessage('assistant', '');
      }

      assistantMessage.text(`${assistantMessage.text()}${event.data}`);
    });
    eventSource.addEventListener('completed', function () {
      finishTurn('Ready for your next message.');
    });
    eventSource.addEventListener('failed', function () {
      finishTurn('Kermet could not complete this turn.');
    });
  }

  function connectExistingThread() {
    const existingThreadId = $('#thread-chat').attr('data-thread-id');

    if (!existingThreadId) {
      return;
    }

    threadId = existingThreadId;
    connectThreadEvents(threadId, function () {
      $threadStatus.text('Ready for your next message.');
    });
  }

  async function sendMessage(message) {
    setTurnPending('Sending message…');
    appendMessage('user', message);
    $threadMessage.val('');

    try {
      const response = await window.fetch(`/api/threads/${threadId}/messages`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json', 'X-Kermet-CSRF': $workspaceCsrfToken.val() },
        body: JSON.stringify({ message: message }),
      });

      if (!response.ok) {
        finishTurn('Kermet could not send the message.');
      }
    } catch (_) {
      finishTurn('Kermet could not send the message.');
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

  $threadsToggle.on('click', toggleThreadsMenu);
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
  $threadForm.on('submit', async function (event) {
    event.preventDefault();
    const workspaceId = $workspaceId.val();
    const message = $threadMessage.val().trim();

    if (!message) {
      $threadStatus.text('Enter a message.');
      return;
    }

    if (threadId !== null) {
      await sendMessage(message);
      return;
    }

    if (!workspaceId) {
      $threadStatus.text('Select a workspace and enter a message.');
      return;
    }

    setTurnPending('Creating thread…');
    $threadStatus.text('Creating thread…');

    try {
      const response = await window.fetch('/api/threads', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json', 'X-Kermet-CSRF': $workspaceCsrfToken.val() },
        body: JSON.stringify({ workspace_id: workspaceId, message: message }),
      });
      if (!response.ok) {
        finishTurn('Kermet could not create the thread.');
        return;
      }
      const thread = await response.json();
      threadId = thread.id;
      window.history.replaceState(null, '', `/threads/${thread.id}`);
      appendMessage('user', message);
      $threadMessage.val('');
      connectThreadEvents(thread.id, function () {
        startInitialTurn();
      });
    } catch (_) {
      finishTurn('Kermet could not create the thread.');
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
