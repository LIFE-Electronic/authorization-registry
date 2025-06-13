import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import "./index.css";
import "@fontsource-variable/merriweather-sans";
import "@fontsource-variable/karla";
import "@fontsource-variable/inter";

import { CssVarsProvider, extendTheme } from "@mui/joy";

import { routeTree } from "./routeTree.gen";
import { AuthProvider } from "./auth";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

// Create a new router instance
const router = createRouter({ routeTree });

const queryClient = new QueryClient();

// Register the router instance for type safety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const theme = extendTheme({
  fontFamily: {
    display: "Merriweather Sans Variable", // applies to `h1`–`h4`
    body: "Karla Variable", // applies to `title-*` and `body-*`
  },
  typography: {
    h2: {
      fontSize: "32px",
    },
    h3: {
      color: "#363D44",
    },
    "body-lg": {
      fontSize: "16px",
      color: "#363D44",
      fontWeight: 400,
    },
    "body-md": {
      fontWeight: 300,
      fontSize: "14px",
      letterSpacing: "-0.43px",
      lineHeight: "16.52px",
      color: "#6D7A88",
    },
    "title-lg": {
      fontSize: "14px",
      color: "#363D44",
      fontFamily: "Merriweather Sans Variable",
    },
    "title-md": {
      fontWeight: 400,
      textTransform: "uppercase",
      letterSpacing: "0px",
      fontSize: "11px",
      color: "#828F9B",
      fontFamily: "Merriweather Sans Variable",
    },
  },
  colorSchemes: {
    light: {
      palette: {
        primary: {
          500: "#4890DA",
        },
        neutral: {
          100: "#DDE0E4",
          300: "#B0B8BF",
          500: "#828F9B",
          600: "#6D7A88",
          700: "#5B6671",
          800: "#49525B",
          900: "#363D44",
          plainHoverBg: "#f6f6f6",
        },
        success: {
          500: "#E1F9DD",
          700: "#349878",
        },
        danger: {
          500: "#D63230",
        },
      },
    },
  },
  components: {
    JoyModal: {
      styleOverrides: {
        backdrop: {
          backgroundColor: "#212529",

          opacity: 0.6,
        },
      },
    },
    JoyModalOverflow: {
      styleOverrides: {
        root: {
          height: "unset",
        },
      },
    },
    JoyChip: {
      styleOverrides: {
        // root: () => ({
        //   fontFamily: "Karla Variable",
        //   paddingLeft: "16px",
        //   paddingRight: "16px",
        //   paddingTop: "8px",
        //   paddingBottom: "8px",
        //   lineHeight: "16.52px",
        //   letterSpacing: "1.1px",
        //   textTransform: "uppercase",
        //   fontWeight: 500,
        //   fontSize: "14px",
        // }),
        startDecorator: {
          paddingRight: 4,
        },
        label: () => ({
          whiteSpace: "pre",
        }),
      },
    },
    JoyOption: {
      styleOverrides: {
        root: () => ({
          fontSize: "16px",
          fontWeight: 400,
        }),
      },
    },
    JoyInput: {
      styleOverrides: {
        root: ({ theme }) => ({
          height: "48px",
          borderColor: theme.vars.palette.neutral[200],
          borderRadius: "8px",
          fontSize: "16px",
          fontWeight: 400,
          color: theme.vars.palette.neutral[900],
          boxShadow: "none",
        }),
      },
    },
    JoySelect: {
      styleOverrides: {
        root: ({ theme }) => ({
          height: "48px",
          borderColor: theme.vars.palette.neutral[200],
          borderRadius: "8px",
          fontSize: "16px",
          fontWeight: 400,
          color: theme.vars.palette.neutral[900],
          boxShadow: "none",
          "&:hover": {
            backgroundColor: "#f6f6f6",
          },
        }),
      },
    },
    JoyAutocomplete: {
      styleOverrides: {
        root: ({ theme }) => ({
          height: "48px",
          borderColor: theme.vars.palette.neutral[200],
          borderRadius: "8px",
          fontSize: "16px",
          fontWeight: 400,
          color: theme.vars.palette.neutral[900],
          boxShadow: "none",
          "&:hover": {
            backgroundColor: "#f6f6f6",
          },
        }),
      },
    },
    JoyFormLabel: {
      styleOverrides: {
        root: () => ({
          fontFamily: "Inter Variable",
          fontSize: "14px",
          fontWeight: 600,
          lineHeight: "24px",
          letterSpacing: "-0.43px",
          color: "#212529",
        }),
      },
    },
    JoyFormHelperText: {
      styleOverrides: {
        root: {
          fontSize: "14px",
          letterSpacing: "-0.43px",
          color: "#212529",
          fontWeight: 400,
          lineHeight: "24px",
        },
      },
    },
    JoyButton: {
      styleOverrides: {
        root: ({ ownerState }) => ({
          minHeight: "14px",
          borderRadius: "8px",
          fontFamily: "Karla Variable",
          paddingLeft: "16px",
          paddingRight: "16px",
          lineHeight: "16.52px",
          paddingTop: "8px",
          paddingBottom: "8px",
          letterSpacing: "1.1px",
          textTransform: "uppercase",
          fontWeight: 500,
          fontSize: "14px",
          ...(ownerState.size === "lg"
            ? {
                paddingLeft: "20px",
                paddingRight: "20px",
                minHeight: "37px",
                paddingTop: "12px",
                paddingBottom: "12px",
              }
            : {}),
          ...(ownerState.size === "sm"
            ? {
                paddingLeft: "20px",
                paddingRight: "20px",
                height: "32px",
                paddingTop: "10px",
                paddingBottom: "10px",
              }
            : {}),
        }),
      },
    },
  },
});

function WrappedApp() {
  return (
    <CssVarsProvider theme={theme}>
      <QueryClientProvider client={queryClient}>
        <AuthProvider>
          <RouterProvider router={router} />
        </AuthProvider>
      </QueryClientProvider>
    </CssVarsProvider>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <WrappedApp />
  </StrictMode>,
);
