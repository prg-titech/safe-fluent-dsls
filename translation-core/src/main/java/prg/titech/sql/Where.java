package prg.titech.sql;

import jakarta.annotation.Nonnull;

public record Where(
        String raw
) {
    public static WhereBuilder columnId(String id) {
        return new WhereBuilder(id);
    }

    @Override
    @Nonnull
    public String toString() {
        return raw;
    }
}
