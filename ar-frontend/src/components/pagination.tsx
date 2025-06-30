import { Box, Button, IconButton } from "@mui/joy";
import ArrowBackIosIcon from "@mui/icons-material/ArrowBackIos";
import ArrowForwardIosIcon from "@mui/icons-material/ArrowForwardIos";

import usePagination from "@mui/material/usePagination";

export function Pagination({
  numberOfItems,
  itemsOnPage,
  page,
  onPageChange,
}: {
  page: number;
  numberOfItems: number;
  onPageChange: (page: number) => void;
  itemsOnPage: number;
}) {
  const numberOfPages = Math.ceil(numberOfItems / itemsOnPage);
  const { items } = usePagination({
    count: numberOfPages,
    page,
    siblingCount: 2,
  });

  return (
    <nav>
      <Box display="flex" alignItems="center">
        {items.map((item, index) => {
          if (item.type === "start-ellipsis" || item.type === "end-ellipsis") {
            return <span key={index}>…</span>;
          }

          if (item.type === "previous") {
            return (
              <IconButton
                size="sm"
                onClick={() => onPageChange(page - 1)}
                key={index}
                disabled={item.disabled}
              >
                <ArrowBackIosIcon />
              </IconButton>
            );
          }

          if (item.type === "next") {
            return (
              <IconButton
                size="sm"
                onClick={() => onPageChange(page + 1)}
                key={index}
                disabled={item.disabled}
              >
                <ArrowForwardIosIcon />
              </IconButton>
            );
          }

          if (item.type === "page") {
            return (
              <Button
                onClick={() => item.page !== null && onPageChange(item.page)}
                key={index}
                variant={item.selected ? "solid" : "plain"}
                color="neutral"
                sx={
                  item.selected
                    ? {
                        backgroundColor: "#363D44",
                      }
                    : {}
                }
              >
                {item.page}
              </Button>
            );
          }
        })}
      </Box>
    </nav>
  );
}
