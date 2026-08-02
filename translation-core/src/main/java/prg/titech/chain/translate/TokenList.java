package prg.titech.chain.translate;

import jakarta.annotation.Nonnull;

import java.awt.*;
import java.util.*;
import java.util.List;
import java.util.stream.Collectors;

public class TokenList implements List<Token> {
    private final String delimiter;
    private final List<Token> base;

    private Token currentToken = new Token("_IGNORE_", -1, 0, new Point(-1, 0), new Point(-1, 0));

    public TokenList() {
        this(" ", new ArrayList<>());
    }

    public TokenList(List<Token> base) {
        this(" ", base);
    }

    public TokenList(String delimiter, List<Token> base) {
        this.delimiter = delimiter;
        this.base = base;
    }

    private Point endOf(String s) {
        String[] parts = s.split("\n", -1); // Negative argument means include empty trailing strings
        int y = parts.length - 1;
        int x = parts[parts.length - 1].getBytes().length;
        return new Point(x, y);
    }

    private Token nextToken(String token) {
        int startByte = currentToken.endByte();
        int endByte = startByte + token.getBytes().length;
        Point start = (Point) currentToken.end().clone();
        start.translate(1, 0);
        Point endOffset = endOf(token);
        Point end;
        if (endOffset.y == 0) {
            end = new Point(start.x + endOffset.x, start.y);
        } else {
            end = new Point(endOffset.x, start.y + endOffset.y);
        }
        currentToken = new Token(token, startByte, endByte, start, end);
        return currentToken;
    }

    public boolean add(String token) {
        add(nextToken(token));
        return add(nextToken(delimiter));
    }

    @Override
    @Nonnull
    public String toString() {
        return stream().map(Object::toString).collect(Collectors.joining());
    }

    public String toDebugString() {
        return "[" + this.base.stream().map(Token::toDebugString).collect(Collectors.joining(", ")) + "]";
    }

    @Override
    public int size() {
        return base.size();
    }

    @Override
    public boolean isEmpty() {
        return base.isEmpty();
    }

    @Override
    public boolean contains(Object o) {
        return base.contains(o);
    }

    @Override
    @Nonnull
    public Iterator<Token> iterator() {
        return base.iterator();
    }

    @Override
    @Nonnull
    public Object[] toArray() {
        return base.toArray();
    }

    @Override
    @Nonnull
    public <T> T[] toArray(@Nonnull T[] a) {
        return base.toArray(a);
    }

    @Override
    public boolean add(Token token) {
        return base.add(token);
    }

    @Override
    public boolean remove(Object o) {
        return base.remove(o);
    }

    @Override
    public boolean containsAll(@Nonnull Collection<?> c) {
        return new HashSet<>(base).containsAll(c);
    }

    @Override
    public boolean addAll(@Nonnull Collection<? extends Token> c) {
        return base.addAll(c);
    }

    @Override
    public boolean addAll(int index, @Nonnull Collection<? extends Token> c) {
        return base.addAll(index, c);
    }

    @Override
    public boolean removeAll(@Nonnull Collection<?> c) {
        return base.removeAll(c);
    }

    @Override
    public boolean retainAll(@Nonnull Collection<?> c) {
        return base.retainAll(c);
    }

    @Override
    public void clear() {
        base.clear();
    }

    @Override
    public Token get(int index) {
        return base.get(index);
    }

    @Override
    public Token set(int index, Token element) {
        return base.set(index, element);
    }

    @Override
    public void add(int index, Token element) {
        base.add(index, element);
    }

    @Override
    public Token remove(int index) {
        return base.remove(index);
    }

    @Override
    public int indexOf(Object o) {
        return base.indexOf(o);
    }

    @Override
    public int lastIndexOf(Object o) {
        return base.lastIndexOf(o);
    }

    @Override
    @Nonnull
    public ListIterator<Token> listIterator() {
        return base.listIterator();
    }

    @Override
    @Nonnull
    public ListIterator<Token> listIterator(int index) {
        return base.listIterator(index);
    }

    @Override
    @Nonnull
    public List<Token> subList(int fromIndex, int toIndex) {
        return base.subList(fromIndex, toIndex);
    }
}
