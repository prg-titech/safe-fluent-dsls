package prg.titech.cli;

import prg.titech.chain.token.Position;
import prg.titech.chain.token.Range;
import prg.titech.chain.token.Token;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardWatchEventKinds;
import java.nio.file.WatchService;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReadWriteLock;
import java.util.concurrent.locks.ReentrantReadWriteLock;

public class SourceFile implements AutoCloseable {
    private final Path path;
    private final FileMonitor fileMonitor;
    private final List<String> latestContent;
    private final List<FileObserver<Object>> observers;
    private final ReadWriteLock lock;
    private Lock readLock;

    public SourceFile(Path path, FileMonitor fileMonitor, List<String> latestContent, List<FileObserver<Object>> observers, ReadWriteLock lock) {
        this.path = path;
        this.fileMonitor = fileMonitor;
        this.latestContent = latestContent;
        this.observers = observers;
        this.lock = lock;
    }

    public static SourceFile fromFile(File file) throws IOException {
        assert(file.isFile()) : "Expected a file but got " + file;
        Path filePath = file.toPath().toAbsolutePath();
        Path dirPath = filePath.getParent();
        WatchService watchService = dirPath.getFileSystem().newWatchService();
        dirPath.register(watchService, StandardWatchEventKinds.ENTRY_MODIFY);
        List<String> latestContent = new ArrayList<>(Files.readAllLines(filePath));
        ReadWriteLock lock = new ReentrantReadWriteLock(true);
        List<FileObserver<Object>> observers = new CopyOnWriteArrayList<>(List.of(
                $ -> {
                    try {
                        List<String> newLines = Files.readAllLines(filePath);
                        Lock readLock = lock.readLock();
                        readLock.lock();
                        boolean wasModified = latestContent.size() != newLines.size();
                        if (!wasModified) {
                            for (int i = 0; i < latestContent.size(); i++) {
                                if (!latestContent.get(i).equals(newLines.get(i))) {
                                    wasModified = true;
                                    break;
                                }
                            }
                        }
                        readLock.unlock();

                        if (wasModified) {
                            Lock writeLock = lock.writeLock();
                            writeLock.lock();
                            latestContent.clear();
                            latestContent.addAll(newLines);
                            writeLock.unlock();
                        }
                    } catch (IOException e) {
                        System.err.println("IOException occurred during source file update: " + e + ". Skipping...");
                    }
                    return $;
                }
        ));

        FileMonitor monitor = new FileMonitor(watchService, 50, observers);
        return new SourceFile(filePath, monitor, latestContent, observers, lock);
    }

    public void addObserver(FileObserver<Object> observer) {
        this.observers.add(observer);
    }

    public String rangeToString(Range range) {
        Lock readLock = lock.readLock();
        readLock.lock();
        String result = latestContent
                .get(range.begin().line() - Position.HOME.line())
                .substring(range.begin().column() - Position.HOME.column(), range.end().column());
        readLock.unlock();
        return result;
    }

    public Token rangeToToken(Range range) {
        String image = rangeToString(range);
        return new Token(image, range);
    }

    @Override
    public String toString() {
        Lock readLock = lock.readLock();
        readLock.lock();
        String result = String.join("\n", latestContent);
        readLock.unlock();
        return result;
    }

    public List<String> getLines() {
        release();
        readLock = lock.readLock();
        readLock.lock();
        return latestContent;
    }

    public void release() {
        if (readLock != null) {
            readLock.unlock();
            readLock = null;
        }
    }

    @Override
    public void close() throws Exception {
        fileMonitor.close();
    }
}
