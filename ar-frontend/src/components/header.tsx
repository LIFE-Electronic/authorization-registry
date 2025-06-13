import { Box, Button, ButtonProps, Typography } from "@mui/joy";
import { Logo } from "./logo";
import { SlashIcon } from "@/icons/slash-icon";
import { KeyIcon } from "@/icons/key-icon";
import { initLogout } from "@/network/idp";
import { ReactNode } from "react";
import { Link } from "@tanstack/react-router";

export function HeaderLink(props: ButtonProps & { selected: boolean }) {
  return (
    <Button
      variant="plain"
      color="neutral"
      sx={{
        typography: {
          color: props.selected ? "#4890DA" : "#49525B",
          fontWeight: 300,
        },
      }}
      {...props}
    />
  );
}

export function Header({ children }: { children?: ReactNode }) {
  return (
    <Box
      position="sticky"
      top={0}
      width="100%"
      sx={{ backgroundColor: "#f4f5f6" }}
      zIndex={1}
    >
      <Box paddingX={2} paddingTop={2}>
        <Box
          display="flex"
          alignItems="center"
          justifyContent="space-between"
          flexGrow={1}
          paddingLeft={3}
          paddingRight={3}
          paddingY={1}
          sx={{ backgroundColor: "white", borderRadius: "64px" }}
        >
          <Box display="flex" alignItems="center" gap={1}>
            <Link to="/">
              <Logo />
            </Link>

            <Box
              width="24px"
              height="24px"
              display="flex"
              alignItems="center"
              justifyContent="center"
            >
              <SlashIcon />
            </Box>

            <Box
              sx={{
                height: "40px",
                display: "flex",
                justifyContent: "center",
                alignItems: "center",
                backgroundColor: "#f4f5f6",
                borderRadius: "4px",
              }}
            >
              <Box
                gap={1}
                display="flex"
                flexDirection="row"
                alignItems="center"
                padding={1}
              >
                <KeyIcon />
                <Typography
                  fontWeight={400}
                  fontSize="16px"
                  textColor="#3602A7"
                >
                  Authorization registry
                </Typography>
              </Box>
            </Box>

            <Box paddingLeft={4}>{children}</Box>
          </Box>
          <Button variant="soft" onClick={() => initLogout()}>
            Logout
          </Button>
        </Box>
      </Box>
    </Box>
  );
}
