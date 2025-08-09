import test from 'ava';
import {SystemTray} from "../index";
test('ok', (t) => {
  const tray = new SystemTray();
  t.truthy(tray);
});