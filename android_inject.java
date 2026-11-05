// Código que cargo-quad-apk inyecta en la MainActivity de miniquad.
// Volumen dentro del juego, acelerómetro para el giroscopio y compartir imagen.
//% IMPORTS
import android.media.AudioManager;
import android.hardware.Sensor;
import android.hardware.SensorEvent;
import android.hardware.SensorEventListener;
import android.hardware.SensorManager;
import android.content.ContentValues;
import android.net.Uri;
import android.provider.MediaStore;
import java.io.FileInputStream;
import java.io.OutputStream;
//% END
//% MAIN_ACTIVITY_ON_CREATE
setVolumeControlStream(AudioManager.STREAM_MUSIC);
bubuSm = (SensorManager) getSystemService(Context.SENSOR_SERVICE);
//% END
//% MAIN_ACTIVITY_ON_RESUME
if (bubuSm != null) {
    Sensor acc = bubuSm.getDefaultSensor(Sensor.TYPE_ACCELEROMETER);
    if (acc != null) bubuSm.registerListener(bubuSl, acc, SensorManager.SENSOR_DELAY_GAME);
}
//% END
//% MAIN_ACTIVITY_ON_PAUSE
if (bubuSm != null) bubuSm.unregisterListener(bubuSl);
//% END
//% MAIN_ACTIVITY_BODY
private SensorManager bubuSm;
private volatile float bubuTiltX = 0f;
private final SensorEventListener bubuSl = new SensorEventListener() {
    @Override
    public void onSensorChanged(SensorEvent e) { bubuTiltX = e.values[0]; }
    @Override
    public void onAccuracyChanged(Sensor s, int a) {}
};

public float getTiltX() { return bubuTiltX; }

// copia el PNG a Imágenes/Bubu Jump y abre el menú de compartir
public boolean shareImage(String path) {
    try {
        // Android 9 o menos pide permiso para guardar en la galería
        if (Build.VERSION.SDK_INT >= 23 && Build.VERSION.SDK_INT < 29
                && checkSelfPermission("android.permission.WRITE_EXTERNAL_STORAGE") != android.content.pm.PackageManager.PERMISSION_GRANTED) {
            requestPermissions(new String[] { "android.permission.WRITE_EXTERNAL_STORAGE" }, 7);
            return false;
        }
        ContentValues cv = new ContentValues();
        cv.put(MediaStore.Images.Media.DISPLAY_NAME, "bubu_jump_" + System.currentTimeMillis() + ".png");
        cv.put(MediaStore.Images.Media.MIME_TYPE, "image/png");
        if (Build.VERSION.SDK_INT >= 29) cv.put(MediaStore.Images.Media.RELATIVE_PATH, "Pictures/Bubu Jump");
        Uri uri = getContentResolver().insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, cv);
        if (uri == null) return false;
        OutputStream out = getContentResolver().openOutputStream(uri);
        FileInputStream in = new FileInputStream(path);
        byte[] buf = new byte[16384];
        int n;
        while ((n = in.read(buf)) > 0) out.write(buf, 0, n);
        in.close();
        out.close();
        final Intent it = new Intent(Intent.ACTION_SEND);
        it.setType("image/png");
        it.putExtra(Intent.EXTRA_STREAM, uri);
        it.putExtra(Intent.EXTRA_TEXT, "¿me superas en Bubu Jump?");
        it.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
        runOnUiThread(new Runnable() {
            @Override
            public void run() { startActivity(Intent.createChooser(it, "compartir")); }
        });
        return true;
    } catch (Exception e) {
        Log.e("BUBU", "shareImage", e);
        return false;
    }
}

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
