package prg.titech.cli;

import java.nio.file.WatchKey;
import java.nio.file.WatchService;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class FileMonitor implements AutoCloseable {
    private final WatchService watchService;
    private final Thread listenerThread;

    public FileMonitor(WatchService watchService, long interval, List<FileObserver<Object>> observers) {
        this.watchService = watchService;
        this.listenerThread = new Thread(() -> {
            Map<FileObserver<Object>, Object> previousOutputs = new HashMap<>();
            try {
                while (true) {
                    Thread.sleep(interval);
                    WatchKey currentKey = watchService.poll();
                    if (currentKey != null) {
                        for (FileObserver<Object> observer : observers) {
                            previousOutputs.compute(observer,
                                    (k, previousOutput) -> observer.notifyUpdate(previousOutput));
                        }
                        currentKey.pollEvents();
                        currentKey.reset();
                    }
                }
            } catch (InterruptedException e) {
                System.out.println("Gracefully shutting down file observer...");
                WatchKey currentKey = watchService.poll();
                if (currentKey != null) {
                    for (FileObserver<Object> observer : observers) {
                        previousOutputs.compute(observer,
                                (k, previousOutput) -> observer.notifyUpdate(previousOutput));
                    }
                }
            }
        });
        listenerThread.start();
    }

    @Override
    public void close() throws Exception {
        watchService.close();
        listenerThread.interrupt();
    }
}
