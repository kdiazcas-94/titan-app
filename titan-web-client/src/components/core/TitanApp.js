import React from 'react'
import PropTypes from 'prop-types'
import { BrowserRouter, Route, Switch } from 'react-router-dom'
import { Provider } from 'react-redux'
import { ThemeProvider } from 'styled-components'
import defaultTheme from 'titan/themes/default'
import AuthenticatedRoute from './routes/AuthenticatedRoute'
import UnauthenticatedRoute from './routes/UnauthenticatedRoute'
import {
  ROUTE_TYPE_AUTHENTICATED,
  ROUTE_TYPE_UNAUTHENTICATED
} from 'titan/lib/routing'

class TitanApp extends React.Component {
  static getRouteComponent (route) {
    switch (route.type) {
      case ROUTE_TYPE_AUTHENTICATED:
        return AuthenticatedRoute
      case ROUTE_TYPE_UNAUTHENTICATED:
        return UnauthenticatedRoute
      default:
        return null
    }
  }

  renderSceneInLayout (route, RouteComponent) {
    const Layout = route.layout
    const Scene = route.scene

    if (!Scene) {
      throw new Error(`Route ${route.path} does not have prop "scene".`)
    }

    let content
    if (Layout) {
      content = (
        <Layout>
          <Scene context={null} />
        </Layout>
      )
    } else {
      content = (<Scene context={null} />)
    }

    if (RouteComponent) {
      return () => (
        <RouteComponent context={null}>
          {content}
        </RouteComponent>
      )
    }

    return () => (content)
  }

  renderRoutes (routes) {
    let routeComponents = []

    Object.keys(routes).forEach((path, key) => {
      const RouteComponent = TitanApp.getRouteComponent(routes[path])
      const Scene = this.renderSceneInLayout(routes[path], RouteComponent)
      const route = (<Route key={key} exact path={path} component={Scene} />)

      routeComponents.push(route)
    })

    return routeComponents
  }

  render () {
    return (
      <ThemeProvider theme={defaultTheme}>
        <Provider store={this.props.context.getStore()}>
          <BrowserRouter>
            <Switch>
              {this.renderRoutes(this.props.context.getRoutes())}
            </Switch>
          </BrowserRouter>
        </Provider>
      </ThemeProvider>
    )
  }
}

TitanApp.propTypes = {
  context: PropTypes.object.isRequired
}

export default TitanApp
