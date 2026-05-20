let pushUndoCallback: ((action: any) => void) | null = null;

export function setPushUndoCallback(cb: (action: any) => void) {
  pushUndoCallback = cb;
}

export function getPushUndoCallback() {
  return pushUndoCallback;
}
