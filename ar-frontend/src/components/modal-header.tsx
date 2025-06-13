import { Box, Typography } from "@mui/joy";
import { KeyIcon } from "@/icons/key-icon";
import { Caption } from "./extra-typography";

export function ModalHeader({
  caption,
  title,
}: {
  caption: string;
  title: string;
}) {
  return (
    <Box
      sx={(theme) => ({
        borderBottomStyle: "solid",
        borderColor: theme.vars.palette.neutral[100],
        borderWidth: "1px",
      })}
      display="flex"
      alignItems="center"
      gap={2}
      padding={2}
    >
      <Box
        sx={(theme) => ({
          backgroundColor: theme.vars.palette.neutral[800],
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          width: "48px",
          height: "48px",
          borderRadius: "48px",
        })}
      >
        <KeyIcon color="white" />
      </Box>
      <Box>
        <Caption>{caption}</Caption>
        <Typography
          fontSize="24px"
          letterSpacing={0}
          fontWeight={700}
          fontFamily="Merriweather Sans Variable"
          lineHeight="118%"
          textColor="neutral.900"
        >
          {title}
        </Typography>
      </Box>
    </Box>
  );
}
