package prg.titech.cli;

@FunctionalInterface
public interface FileObserver<S> {

    S notifyUpdate(S state);

}
