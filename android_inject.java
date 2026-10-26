// Código que cargo-quad-apk inyecta en la MainActivity de miniquad.
// Hace que los botones de volumen funcionen dentro del juego.
//% IMPORTS
import android.media.AudioManager;
//% END
//% MAIN_ACTIVITY_ON_CREATE
setVolumeControlStream(AudioManager.STREAM_MUSIC);
//% END
//% MAIN_ACTIVITY_BODY
@Override
public boolean dispatchKeyEvent(KeyEvent e) {
    int k = e.getKeyCode();
    if (k == KeyEvent.KEYCODE_VOLUME_UP || k == KeyEvent.KEYCODE_VOLUME_DOWN || k == KeyEvent.KEYCODE_VOLUME_MUTE) {
        if (e.getAction() == KeyEvent.ACTION_DOWN) {
            AudioManager am = (AudioManager) getSystemService(AUDIO_SERVICE);
            int dir = k == KeyEvent.KEYCODE_VOLUME_UP ? AudioManager.ADJUST_RAISE
                : (k == KeyEvent.KEYCODE_VOLUME_DOWN ? AudioManager.ADJUST_LOWER : AudioManager.ADJUST_TOGGLE_MUTE);
            am.adjustStreamVolume(AudioManager.STREAM_MUSIC, dir, AudioManager.FLAG_SHOW_UI);
        }
        return true;
    }
    return super.dispatchKeyEvent(e);
}
//% END
